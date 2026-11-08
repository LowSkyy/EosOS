//! Backend `Internal` dei desktop virtuali (Mission Control, fase M5): le interfacce NON documentate di
//! Windows (`IVirtualDesktopManagerInternal` e compagne), usate solo se superano una verifica di coerenza.
//!
//! Resilienza, come da specifica:
//! - **Profili guidati da dati** (`vdprofile`): IID e indici dei metodi stanno in un file, uno incorporato e uno
//!   aggiornabile dall'utente; la chiamata dei metodi avviene per indice nella tabella virtuale.
//! - **Selezione per build con prova**: si provano prima i profili che dichiarano la build, poi gli altri dal più
//!   vicino; ciascuno deve superare la **verifica di coerenza**, fatta SOLO con metodi di lettura (numero e
//!   identificatori dei desktop, desktop corrente, ricerca per id, desktop adiacente, elenco delle viste) e
//!   confrontata con il registro, che è la fonte indipendente. Un profilo che non la supera non si usa.
//! - **Degradazione a gradini**: senza un profilo valido restano elenco, passaggio e creazione dei desktop
//!   (`vdesk`, solo documentato); spostare finestre, rimuovere un desktop qualsiasi e rinominare risultano
//!   non disponibili, con il motivo nel rapporto.
//! - **Rapporto diagnostico** (build, profili provati, esito di ogni controllo; nessun dato dell'utente).
//! - **Protezione dai crash ripetuti**: una chiamata sulla tabella virtuale di un'interfaccia con layout diverso
//!   può terminare il processo; prima di ogni tentativo si scrive un segnale su disco e lo si toglie a
//!   tentativo finito. Se all'avvio il segnale c'è, il tentativo precedente è finito male: il backend interno
//!   resta spento finché non si cancella il segnale (o si aggiornano i profili) invece di ripetere il crash.
//!
//! Le operazioni che modificano qualcosa (`move_window`, `remove_desktop`, `rename_desktop`) esistono solo
//! con un profilo che ha superato la verifica. I loro indici non si possono provare senza modificare davvero
//! lo stato: sono "vicini" a quelli verificati (stesso blocco della tabella), e la prova che li esercita è
//! `scripts\test-vdesk-internal.ps1`, da eseguire solo con l'ok esplicito dell'utente.

use std::ffi::c_void;
use std::path::PathBuf;
use std::sync::Mutex;

use serde::Serialize;
use windows::core::{Interface, GUID, HRESULT, HSTRING};
use windows::Win32::Foundation::HWND;
use windows::Win32::System::Com::{
    CoCreateInstance, CoInitializeEx, CoUninitialize, IServiceProvider, CLSCTX_LOCAL_SERVER, COINIT_APARTMENTTHREADED,
};

use crate::vdprofile::{candidates, guid_text, merge, parse_guid, parse_profiles, Profile, BUILTIN};

const CLSID_IMMERSIVE_SHELL: GUID = GUID::from_u128(0xC2F03A33_21F5_47FA_B4BB_156362A2F239);
/// Indice di `QueryService` in `IServiceProvider` (dopo i tre di IUnknown).
const QUERY_SERVICE: u16 = 3;
/// Indici di `IObjectArray`.
const ARRAY_GET_COUNT: u16 = 3;
const ARRAY_GET_AT: u16 = 4;
/// Direzioni di `GetAdjacentDesktop`.
const LEFT: u32 = 3;
const RIGHT: u32 = 4;

// ------------------------------------------------------------------ tipi del rapporto

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
pub struct Check {
    pub name: String,
    pub ok: bool,
    pub detail: String,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
pub struct Attempt {
    pub profile: String,
    /// Il profilo dichiara di valere per questa build (altrimenti è un tentativo "per vicinanza").
    pub declared: bool,
    pub passed: bool,
    pub checks: Vec<Check>,
}

/// Cosa si può fare con il backend scelto.
#[derive(Serialize, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Caps {
    /// Un profilo ha superato la verifica: le interfacce non documentate sono usabili.
    pub internal: bool,
    /// Spostare una finestra su un altro desktop.
    pub move_window: bool,
    /// Rimuovere un desktop qualunque (senza `internal` solo il corrente, con Win+Ctrl+F4).
    pub remove_any: bool,
    /// Rinominare un desktop.
    pub rename: bool,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
pub struct Report {
    pub build: u32,
    pub revision: u32,
    pub display_version: String,
    pub profiles_loaded: usize,
    /// Esito della lettura del file dei profili dell'utente ("assente", "ok", o l'errore).
    pub user_profiles: String,
    pub problems: Vec<String>,
    pub attempts: Vec<Attempt>,
    pub selected: Option<String>,
    /// Perché il backend interno non è attivo, se non lo è.
    pub reason: Option<String>,
    pub caps: Caps,
}

// ------------------------------------------------------------------ parti pure

/// Cosa fare del segnale di crash e del permesso di provare. Pura, testata.
pub fn may_try(flag_exists: bool) -> Result<(), String> {
    if flag_exists {
        Err("un tentativo precedente e' terminato in modo anomalo: il backend interno resta spento. Dopo aver aggiornato i profili cancella vdesk-crash.flag in %LOCALAPPDATA%\\EosOS per riprovare".into())
    } else {
        Ok(())
    }
}

/// Dalla riuscita dei tentativi e dal profilo scelto alle capacità. Pura, testata.
pub fn caps_for(selected: bool) -> Caps {
    Caps { internal: selected, move_window: selected, remove_any: selected, rename: selected }
}

/// Il motivo per cui il backend interno non è attivo, a partire dai tentativi. Pura, testata.
pub fn reason_for(attempts: &[Attempt], profiles_loaded: usize, crash: Option<&str>) -> Option<String> {
    if let Some(c) = crash {
        return Some(c.to_string());
    }
    if profiles_loaded == 0 {
        return Some("nessun profilo valido caricato".into());
    }
    if attempts.iter().any(|a| a.passed) {
        return None;
    }
    let first_failed = attempts.iter().flat_map(|a| a.checks.iter().map(move |c| (a, c))).find(|(_, c)| !c.ok);
    Some(match first_failed {
        Some((a, c)) => format!("nessun profilo supera la verifica (per esempio {}: {} - {})", a.profile, c.name, c.detail),
        None => "nessun profilo provato".into(),
    })
}

/// Confronta due elenchi di id senza badare alle maiuscole e all'ordine di scrittura. Pura, testata.
pub fn same_ids(a: &[String], b: &[String]) -> bool {
    a.len() == b.len() && a.iter().zip(b).all(|(x, y)| x.eq_ignore_ascii_case(y))
}

// ------------------------------------------------------------------ COM per indice di metodo

type Raw = *mut c_void;

/// Il metodo `index` della tabella virtuale di `obj`, come puntatore a funzione di tipo `F`.
///
/// # Safety
/// `obj` deve essere un puntatore COM valido e `F` deve essere il tipo di funzione che quel metodo ha davvero.
unsafe fn slot<F: Copy>(obj: Raw, index: u16) -> F {
    unsafe {
        let vtable = *(obj as *const *const usize);
        let f = *vtable.add(usize::from(index));
        std::mem::transmute_copy::<usize, F>(&f)
    }
}

fn ok(hr: HRESULT) -> Result<(), String> {
    if hr.is_ok() { Ok(()) } else { Err(format!("HRESULT 0x{:08X}", hr.0 as u32)) }
}

/// Un oggetto COM posseduto (si rilascia da solo).
struct Obj(windows::core::IUnknown);

impl Obj {
    fn raw(&self) -> Raw {
        self.0.as_raw()
    }

    /// Prende possesso di un puntatore restituito da un metodo (con un riferimento già contato).
    unsafe fn own(p: Raw) -> Option<Obj> {
        if p.is_null() { None } else { Some(Obj(unsafe { windows::core::IUnknown::from_raw(p) })) }
    }
}

/// Il fornitore di servizi della shell, da cui si chiedono i gestori.
fn service_provider() -> Result<IServiceProvider, String> {
    unsafe {
        let unk: windows::core::IUnknown =
            CoCreateInstance(&CLSID_IMMERSIVE_SHELL, None, CLSCTX_LOCAL_SERVER).map_err(|e| format!("ImmersiveShell: {e}"))?;
        unk.cast::<IServiceProvider>().map_err(|e| format!("IServiceProvider: {e}"))
    }
}

unsafe fn query_service(sp: &IServiceProvider, service: &GUID, iid: &GUID) -> Result<Obj, String> {
    unsafe {
        type F = unsafe extern "system" fn(Raw, *const GUID, *const GUID, *mut Raw) -> HRESULT;
        let f: F = slot(sp.as_raw(), QUERY_SERVICE);
        let mut out: Raw = std::ptr::null_mut();
        ok(f(sp.as_raw(), service, iid, &mut out))?;
        Obj::own(out).ok_or_else(|| "interfaccia nulla".to_string())
    }
}

/// Il gestore dei desktop (`IVirtualDesktopManagerInternal`) secondo un profilo.
struct Mgr<'a> {
    obj: Obj,
    p: &'a Profile,
}

/// Un desktop (`IVirtualDesktop`).
struct Desk(Obj);

impl Desk {
    fn id(&self, p: &Profile) -> Result<String, String> {
        unsafe {
            type F = unsafe extern "system" fn(Raw, *mut GUID) -> HRESULT;
            let f: F = slot(self.0.raw(), p.desktop.methods.get_id);
            let mut g = GUID::zeroed();
            ok(f(self.0.raw(), &mut g))?;
            Ok(guid_text(&g))
        }
    }
}

/// Un elenco di oggetti (`IObjectArray`): numero e accesso per indice con un IID scelto a runtime.
unsafe fn array_items(array: Raw, iid: &GUID) -> Result<Vec<Obj>, String> {
    unsafe {
        let holder = Obj::own(array).ok_or("elenco nullo")?;
        type Count = unsafe extern "system" fn(Raw, *mut u32) -> HRESULT;
        type GetAt = unsafe extern "system" fn(Raw, u32, *const GUID, *mut Raw) -> HRESULT;
        let count: Count = slot(holder.raw(), ARRAY_GET_COUNT);
        let get_at: GetAt = slot(holder.raw(), ARRAY_GET_AT);
        let mut n = 0u32;
        ok(count(holder.raw(), &mut n))?;
        if n > 64 {
            return Err(format!("{n} elementi: troppi per essere desktop"));
        }
        let mut out = Vec::new();
        for i in 0..n {
            let mut item: Raw = std::ptr::null_mut();
            ok(get_at(holder.raw(), i, iid, &mut item))?;
            out.push(Obj::own(item).ok_or("elemento nullo")?);
        }
        Ok(out)
    }
}

impl<'a> Mgr<'a> {
    fn open(sp: &IServiceProvider, p: &'a Profile) -> Result<Mgr<'a>, String> {
        let (service, iid) = (parse_guid(&p.manager.service).ok_or("service non valido")?, parse_guid(&p.manager.iid).ok_or("iid non valido")?);
        let obj = unsafe { query_service(sp, &service, &iid)? };
        Ok(Mgr { obj, p })
    }

    fn desk_iid(&self) -> GUID {
        parse_guid(&self.p.desktop.iid).unwrap_or_else(GUID::zeroed)
    }

    fn count(&self) -> Result<u32, String> {
        unsafe {
            let mut n = 0u32;
            let m = self.p.manager.methods.get_count;
            if self.p.monitor_args {
                type F = unsafe extern "system" fn(Raw, isize, *mut u32) -> HRESULT;
                let f: F = slot(self.obj.raw(), m);
                ok(f(self.obj.raw(), 0, &mut n))?;
            } else {
                type F = unsafe extern "system" fn(Raw, *mut u32) -> HRESULT;
                let f: F = slot(self.obj.raw(), m);
                ok(f(self.obj.raw(), &mut n))?;
            }
            Ok(n)
        }
    }

    fn desktops(&self) -> Result<Vec<Desk>, String> {
        unsafe {
            let mut arr: Raw = std::ptr::null_mut();
            let m = self.p.manager.methods.get_desktops;
            if self.p.monitor_args {
                type F = unsafe extern "system" fn(Raw, isize, *mut Raw) -> HRESULT;
                let f: F = slot(self.obj.raw(), m);
                ok(f(self.obj.raw(), 0, &mut arr))?;
            } else {
                type F = unsafe extern "system" fn(Raw, *mut Raw) -> HRESULT;
                let f: F = slot(self.obj.raw(), m);
                ok(f(self.obj.raw(), &mut arr))?;
            }
            Ok(array_items(arr, &self.desk_iid())?.into_iter().map(Desk).collect())
        }
    }

    fn current(&self) -> Result<Desk, String> {
        unsafe {
            let mut d: Raw = std::ptr::null_mut();
            let m = self.p.manager.methods.get_current;
            if self.p.monitor_args {
                type F = unsafe extern "system" fn(Raw, isize, *mut Raw) -> HRESULT;
                let f: F = slot(self.obj.raw(), m);
                ok(f(self.obj.raw(), 0, &mut d))?;
            } else {
                type F = unsafe extern "system" fn(Raw, *mut Raw) -> HRESULT;
                let f: F = slot(self.obj.raw(), m);
                ok(f(self.obj.raw(), &mut d))?;
            }
            Ok(Desk(Obj::own(d).ok_or("desktop nullo")?))
        }
    }

    fn find(&self, id: &GUID) -> Result<Desk, String> {
        unsafe {
            type F = unsafe extern "system" fn(Raw, *const GUID, *mut Raw) -> HRESULT;
            let f: F = slot(self.obj.raw(), self.p.manager.methods.find);
            let mut d: Raw = std::ptr::null_mut();
            ok(f(self.obj.raw(), id, &mut d))?;
            Ok(Desk(Obj::own(d).ok_or("desktop nullo")?))
        }
    }

    fn adjacent(&self, of: &Desk, dir: u32) -> Result<Desk, String> {
        unsafe {
            type F = unsafe extern "system" fn(Raw, Raw, u32, *mut Raw) -> HRESULT;
            let f: F = slot(self.obj.raw(), self.p.manager.methods.get_adjacent);
            let mut d: Raw = std::ptr::null_mut();
            ok(f(self.obj.raw(), of.0.raw(), dir, &mut d))?;
            Ok(Desk(Obj::own(d).ok_or("desktop nullo")?))
        }
    }

    /// `GetDesktopSwitchIncludeExcludeViews`: due elenchi di viste (sola lettura).
    fn include_exclude(&self, of: &Desk) -> Result<(), String> {
        unsafe {
            type F = unsafe extern "system" fn(Raw, Raw, *mut Raw, *mut Raw) -> HRESULT;
            let f: F = slot(self.obj.raw(), self.p.manager.methods.include_exclude);
            let (mut a, mut b): (Raw, Raw) = (std::ptr::null_mut(), std::ptr::null_mut());
            ok(f(self.obj.raw(), of.0.raw(), &mut a, &mut b))?;
            // Gli elenchi sono due IObjectArray: si rilasciano senza leggerli.
            drop(Obj::own(a));
            drop(Obj::own(b));
            Ok(())
        }
    }

    fn can_move_view(&self, view: &Obj) -> Result<i32, String> {
        unsafe {
            type F = unsafe extern "system" fn(Raw, Raw, *mut i32) -> HRESULT;
            let f: F = slot(self.obj.raw(), self.p.manager.methods.can_move_view);
            let mut can = 0i32;
            ok(f(self.obj.raw(), view.raw(), &mut can))?;
            Ok(can)
        }
    }

    // --- operazioni che modificano lo stato (solo con un profilo che ha superato la verifica)

    fn move_view(&self, view: &Obj, to: &Desk) -> Result<(), String> {
        unsafe {
            type F = unsafe extern "system" fn(Raw, Raw, Raw) -> HRESULT;
            let f: F = slot(self.obj.raw(), self.p.manager.methods.move_view);
            ok(f(self.obj.raw(), view.raw(), to.0.raw()))
        }
    }

    fn remove(&self, victim: &Desk, fallback: &Desk) -> Result<(), String> {
        unsafe {
            type F = unsafe extern "system" fn(Raw, Raw, Raw) -> HRESULT;
            let f: F = slot(self.obj.raw(), self.p.manager.methods.remove);
            ok(f(self.obj.raw(), victim.0.raw(), fallback.0.raw()))
        }
    }

    fn set_name(&self, d: &Desk, name: &str) -> Result<(), String> {
        unsafe {
            type F = unsafe extern "system" fn(Raw, Raw, *mut c_void) -> HRESULT;
            let f: F = slot(self.obj.raw(), self.p.manager.methods.set_name);
            let h = HSTRING::from(name);
            ok(f(self.obj.raw(), d.0.raw(), std::mem::transmute_copy::<HSTRING, *mut c_void>(&h)))
        }
    }
}

/// La vista (`IApplicationView`) di una finestra.
fn view_for_hwnd(sp: &IServiceProvider, p: &Profile, hwnd: HWND) -> Result<Obj, String> {
    unsafe {
        let (service, iid) = (parse_guid(&p.views.service).ok_or("service non valido")?, parse_guid(&p.views.iid).ok_or("iid non valido")?);
        let coll = query_service(sp, &service, &iid)?;
        type F = unsafe extern "system" fn(Raw, HWND, *mut Raw) -> HRESULT;
        let f: F = slot(coll.raw(), p.views.methods.get_view_for_hwnd);
        let mut v: Raw = std::ptr::null_mut();
        ok(f(coll.raw(), hwnd, &mut v))?;
        Obj::own(v).ok_or_else(|| "la finestra non ha una vista".to_string())
    }
}

// ------------------------------------------------------------------ verifica di coerenza

fn check(name: &str, r: Result<String, String>) -> Check {
    match r {
        Ok(d) => Check { name: name.into(), ok: true, detail: d },
        Err(e) => Check { name: name.into(), ok: false, detail: e },
    }
}

/// La verifica di coerenza: SOLO metodi di lettura, confrontati con il registro. Si ferma al primo controllo
/// fondamentale che fallisce (i successivi dipenderebbero da un layout che non regge). `probe`: una finestra
/// qualunque per provare l'elenco delle viste, se c'è.
fn selftest(sp: &IServiceProvider, p: &Profile, probe: Option<HWND>) -> Vec<Check> {
    let mut out = Vec::new();
    let reg = crate::vdesk::list();
    let reg_ids: Vec<String> = reg.iter().filter(|d| !d.id.is_empty()).map(|d| d.id.clone()).collect();
    let reg_current = reg.iter().find(|d| d.current).map(|d| d.id.clone()).unwrap_or_default();

    let mgr = match Mgr::open(sp, p) {
        Ok(m) => {
            out.push(check("gestore dei desktop (QueryService con l'IID del profilo)", Ok("risponde".into())));
            m
        }
        Err(e) => {
            out.push(check("gestore dei desktop (QueryService con l'IID del profilo)", Err(e)));
            return out;
        }
    };
    let count = mgr.count();
    let count_ok = matches!(&count, Ok(n) if *n as usize == reg.len());
    out.push(check(
        "numero di desktop = registro",
        count.map(|n| format!("{n}")).and_then(|n| if count_ok { Ok(n) } else { Err(format!("{n} contro {} nel registro", reg.len())) }),
    ));
    if !count_ok {
        return out;
    }
    // Se il registro non ha id (un solo desktop mai toccato) non c'e' nulla da confrontare oltre al numero.
    let list = mgr.desktops();
    let ids: Result<Vec<String>, String> = list.and_then(|v| v.iter().map(|d| d.id(p)).collect());
    let ids_ok = match (&ids, reg_ids.is_empty()) {
        (Ok(v), false) => same_ids(v, &reg_ids),
        (Ok(v), true) => v.len() == 1,
        _ => false,
    };
    out.push(check(
        "elenco dei desktop: stessi id, nello stesso ordine del registro",
        ids.clone().and_then(|v| if ids_ok { Ok(format!("{} desktop", v.len())) } else { Err(format!("{v:?} contro {reg_ids:?}")) }),
    ));
    if !ids_ok {
        return out;
    }
    let cur = mgr.current().and_then(|d| d.id(p));
    let cur_ok = match &cur {
        Ok(c) => reg_current.is_empty() || c.eq_ignore_ascii_case(&reg_current),
        Err(_) => false,
    };
    out.push(check("desktop corrente = registro", cur.clone().and_then(|c| if cur_ok { Ok(c) } else { Err(format!("{c} contro {reg_current}")) })));
    if !cur_ok {
        return out;
    }
    // Ricerca per id: ogni desktop deve tornare con lo stesso id.
    let found: Result<usize, String> = ids.clone().and_then(|v| {
        let mut n = 0;
        for id in &v {
            let g = parse_guid(id).ok_or("id non valido")?;
            let d = mgr.find(&g)?;
            if !d.id(p)?.eq_ignore_ascii_case(id) {
                return Err(format!("find({id}) ha restituito un altro desktop"));
            }
            n += 1;
        }
        Ok(n)
    });
    let found_ok = found.is_ok();
    out.push(check("ricerca per id (find)", found.map(|n| format!("{n} trovati"))));
    if !found_ok {
        return out;
    }
    // Desktop adiacente: coerente con l'ordine del registro (se ce n'e' piu' di uno).
    let all = ids.unwrap_or_default();
    let ci = all.iter().position(|i| i.eq_ignore_ascii_case(&reg_current)).unwrap_or(0);
    let adj: Result<String, String> = (|| {
        let here = mgr.current()?;
        if all.len() < 2 {
            return Ok("un solo desktop: niente da confrontare".to_string());
        }
        let mut seen = 0;
        for (dir, want) in [(LEFT, ci.checked_sub(1)), (RIGHT, if ci + 1 < all.len() { Some(ci + 1) } else { None })] {
            match (mgr.adjacent(&here, dir), want) {
                (Ok(d), Some(w)) if d.id(p)?.eq_ignore_ascii_case(&all[w]) => seen += 1,
                (Ok(d), Some(w)) => return Err(format!("adiacente {} invece di {}", d.id(p)?, all[w])),
                (Err(_), None) => seen += 1, // nessun vicino da quel lato: un errore e' corretto
                (Err(e), Some(_)) => return Err(format!("adiacente non trovato: {e}")),
                (Ok(_), None) => return Err("trovato un vicino dove non dovrebbe essercene".into()),
            }
        }
        Ok(format!("{seen} direzioni coerenti"))
    })();
    let adj_ok = adj.is_ok();
    out.push(check("desktop adiacente (a sinistra e a destra)", adj));
    if !adj_ok {
        return out;
    }
    out.push(check("elenchi di viste del desktop corrente (sola lettura)", mgr.current().and_then(|d| mgr.include_exclude(&d)).map(|_| "due elenchi".into())));
    // Viste: serve una finestra qualunque.
    match probe {
        Some(h) => {
            let view = view_for_hwnd(sp, p, h);
            let view_ok = view.is_ok();
            out.push(check("vista di una finestra (IApplicationViewCollection)", view.as_ref().map(|_| "trovata".to_string()).map_err(Clone::clone)));
            if let (true, Ok(v)) = (view_ok, view) {
                out.push(check("la finestra e' spostabile? (CanViewMoveDesktops, sola lettura)", mgr.can_move_view(&v).map(|c| format!("risposta {c}"))));
            }
        }
        None => out.push(Check { name: "viste di finestre".into(), ok: true, detail: "saltato: nessuna finestra da provare".into() }),
    }
    out
}

// ------------------------------------------------------------------ selezione e stato

struct Selection {
    report: Report,
    profile: Option<Profile>,
}

static SELECTION: Mutex<Option<Selection>> = Mutex::new(None);

fn crash_flag() -> Option<PathBuf> {
    Some(PathBuf::from(std::env::var_os("LOCALAPPDATA")?).join(r"EosOS\vdesk-crash.flag"))
}

fn user_profiles_path() -> Option<PathBuf> {
    Some(PathBuf::from(std::env::var_os("LOCALAPPDATA")?).join(r"EosOS\vdesk-profiles.json"))
}

/// (build, revisione, versione visualizzata) di Windows, dal registro.
pub(crate) fn build_info() -> (u32, u32, String) {
    use windows::core::HSTRING as H;
    use windows::Win32::System::Registry::{RegCloseKey, RegOpenKeyExW, RegQueryValueExW, HKEY, HKEY_LOCAL_MACHINE, KEY_READ, REG_VALUE_TYPE};
    unsafe {
        let mut key = HKEY::default();
        if RegOpenKeyExW(HKEY_LOCAL_MACHINE, &H::from(r"SOFTWARE\Microsoft\Windows NT\CurrentVersion"), None, KEY_READ, &mut key).ok().is_err() {
            return (0, 0, String::new());
        }
        let read = |name: &str| -> Option<Vec<u8>> {
            let n = H::from(name);
            let mut ty = REG_VALUE_TYPE(0);
            let mut len = 0u32;
            RegQueryValueExW(key, &n, None, Some(&mut ty), None, Some(&mut len)).ok().ok()?;
            let mut buf = vec![0u8; len as usize];
            RegQueryValueExW(key, &n, None, Some(&mut ty), Some(buf.as_mut_ptr()), Some(&mut len)).ok().ok()?;
            buf.truncate(len as usize);
            Some(buf)
        };
        let build = read("CurrentBuild").map(|b| crate::vdesk::utf16_to_string(&b)).and_then(|s| s.trim().parse().ok()).unwrap_or(0);
        let ubr = read("UBR").and_then(|b| b.get(..4).map(|x| u32::from_le_bytes([x[0], x[1], x[2], x[3]]))).unwrap_or(0);
        let ver = read("DisplayVersion").map(|b| crate::vdesk::utf16_to_string(&b)).unwrap_or_default();
        let _ = RegCloseKey(key);
        (build, ubr, ver)
    }
}

/// Esegue `f` su un thread con COM inizializzato (apartment a thread singolo) e lo chiude dopo. Una chiamata
/// sulla tabella virtuale è su un thread proprio: un guasto non si porta dietro lo stato degli altri.
fn with_com<R: Send + 'static>(f: impl FnOnce() -> R + Send + 'static) -> Option<R> {
    std::thread::spawn(move || unsafe {
        let init = CoInitializeEx(None, COINIT_APARTMENTTHREADED).is_ok();
        let r = f();
        if init {
            CoUninitialize();
        }
        r
    })
    .join()
    .ok()
}

/// Una finestra qualunque da usare per provare l'elenco delle viste (come numero: gli HWND non attraversano i thread).
fn probe_window() -> Option<usize> {
    crate::dock::raw_app_windows().into_iter().map(|(h, _, _)| h.0 as usize).next()
}

/// Carica i profili (dell'utente e incorporati), li prova in ordine e sceglie il primo che supera la verifica.
fn select() -> Selection {
    let (build, ubr, version) = build_info();
    let (builtin, mut problems) = parse_profiles(BUILTIN).unwrap_or_default();
    let (user, user_status) = match user_profiles_path().and_then(|p| std::fs::read_to_string(p).ok()) {
        None => (Vec::new(), "assente".to_string()),
        Some(text) => match parse_profiles(&text) {
            Ok((ok, bad)) => {
                problems.extend(bad.into_iter().map(|b| format!("file dell'utente: {b}")));
                (ok, "ok".to_string())
            }
            Err(e) => (Vec::new(), e),
        },
    };
    let profiles = merge(user, builtin);
    let mut report = Report {
        build,
        revision: ubr,
        display_version: version,
        profiles_loaded: profiles.len(),
        user_profiles: user_status,
        problems,
        attempts: Vec::new(),
        selected: None,
        reason: None,
        caps: caps_for(false),
    };
    let flag = crash_flag();
    if let Err(why) = may_try(flag.as_ref().is_some_and(|f| f.exists())) {
        report.reason = reason_for(&report.attempts, report.profiles_loaded, Some(&why));
        return Selection { report, profile: None };
    }
    let mut chosen: Option<Profile> = None;
    for (profile, declared) in candidates(&profiles, build, ubr) {
        // Segnale PRIMA di toccare la tabella virtuale; si toglie a fine tentativo.
        if let Some(f) = &flag {
            let _ = std::fs::create_dir_all(f.parent().unwrap_or(f));
            let _ = std::fs::write(f, format!("tentativo con il profilo {}\n", profile.id));
        }
        let p = profile.clone();
        let probe = probe_window();
        let checks = with_com(move || match service_provider() {
            Ok(sp) => selftest(&sp, &p, probe.map(|h| HWND(h as *mut _))),
            Err(e) => vec![Check { name: "servizi della shell".into(), ok: false, detail: e }],
        })
        .unwrap_or_else(|| vec![Check { name: "verifica".into(), ok: false, detail: "il thread della verifica e' terminato in modo anomalo".into() }]);
        if let Some(f) = &flag {
            let _ = std::fs::remove_file(f);
        }
        let passed = !checks.is_empty() && checks.iter().all(|c| c.ok);
        report.attempts.push(Attempt { profile: profile.id.clone(), declared, passed, checks });
        if passed {
            chosen = Some(profile.clone());
            break;
        }
    }
    report.selected = chosen.as_ref().map(|p| p.id.clone());
    report.caps = caps_for(chosen.is_some());
    report.reason = reason_for(&report.attempts, report.profiles_loaded, None);
    Selection { report, profile: chosen }
}

/// Il rapporto corrente; alla prima richiesta (o con `refresh`) fa la selezione, che costa alcuni millisecondi.
pub fn report(refresh: bool) -> Report {
    let mut s = SELECTION.lock().unwrap_or_else(|e| e.into_inner());
    if refresh || s.is_none() {
        *s = Some(select());
    }
    s.as_ref().map(|x| x.report.clone()).unwrap_or_else(|| select().report)
}

fn profile() -> Option<Profile> {
    report(false);
    SELECTION.lock().unwrap_or_else(|e| e.into_inner()).as_ref().and_then(|s| s.profile.clone())
}

/// Cosa si può fare ora.
pub fn caps() -> Caps {
    report(false).caps
}

// ------------------------------------------------------------------ operazioni

/// Sposta la finestra `hwnd` sul desktop numero `index` (indice nell'elenco). Solo con un profilo verificato.
pub fn move_window(hwnd: usize, index: usize) -> Result<(), String> {
    let p = profile().ok_or("backend interno non disponibile")?;
    with_com(move || {
        let sp = service_provider()?;
        let mgr = Mgr::open(&sp, &p)?;
        let desks = mgr.desktops()?;
        let target = desks.get(index).ok_or("desktop inesistente")?;
        let view = view_for_hwnd(&sp, &p, HWND(hwnd as *mut _))?;
        if mgr.can_move_view(&view)? == 0 {
            return Err("Windows non permette di spostare questa finestra".into());
        }
        mgr.move_view(&view, target)
    })
    .unwrap_or_else(|| Err("operazione terminata in modo anomalo".into()))
}

/// Rimuove il desktop `index`: le sue finestre passano al vicino (il precedente, o il successivo se è il primo).
/// Solo con un profilo verificato e con almeno due desktop.
pub fn remove_desktop(index: usize) -> Result<(), String> {
    let p = profile().ok_or("backend interno non disponibile")?;
    with_com(move || {
        let sp = service_provider()?;
        let mgr = Mgr::open(&sp, &p)?;
        let desks = mgr.desktops()?;
        if desks.len() < 2 {
            return Err("l'ultimo desktop non si rimuove".into());
        }
        let victim = desks.get(index).ok_or("desktop inesistente")?;
        let fallback = &desks[if index > 0 { index - 1 } else { 1 }];
        mgr.remove(victim, fallback)
    })
    .unwrap_or_else(|| Err("operazione terminata in modo anomalo".into()))
}

/// Dà un nome al desktop `index` (al massimo 60 caratteri, senza spazi ai bordi). Solo con un profilo verificato.
pub fn rename_desktop(index: usize, name: &str) -> Result<(), String> {
    let name: String = name.trim().chars().take(60).collect();
    let p = profile().ok_or("backend interno non disponibile")?;
    with_com(move || {
        let sp = service_provider()?;
        let mgr = Mgr::open(&sp, &p)?;
        let desks = mgr.desktops()?;
        mgr.set_name(desks.get(index).ok_or("desktop inesistente")?, &name)
    })
    .unwrap_or_else(|| Err("operazione terminata in modo anomalo".into()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn attempt(profile: &str, passed: bool, failing: Option<(&str, &str)>) -> Attempt {
        let mut checks = vec![Check { name: "gestore".into(), ok: true, detail: "risponde".into() }];
        if let Some((n, d)) = failing {
            checks.push(Check { name: n.into(), ok: false, detail: d.into() });
        }
        Attempt { profile: profile.into(), declared: true, passed, checks }
    }

    #[test]
    fn a_crash_flag_disables_the_internal_backend_until_it_is_removed() {
        assert!(may_try(false).is_ok());
        let e = may_try(true).unwrap_err();
        assert!(e.contains("vdesk-crash.flag") && e.contains("anomalo"));
    }

    #[test]
    fn capabilities_follow_the_selection_all_or_nothing() {
        assert_eq!(caps_for(false), Caps::default());
        let c = caps_for(true);
        assert!(c.internal && c.move_window && c.remove_any && c.rename);
    }

    #[test]
    fn the_reason_explains_why_the_internal_backend_is_off() {
        assert_eq!(reason_for(&[attempt("a", true, None)], 1, None), None, "un profilo ha funzionato");
        assert!(reason_for(&[], 0, None).unwrap().contains("nessun profilo valido"));
        assert!(reason_for(&[], 2, None).unwrap().contains("nessun profilo provato"));
        let r = reason_for(&[attempt("w11", false, Some(("numero di desktop = registro", "2 contro 1")))], 1, None).unwrap();
        assert!(r.contains("w11") && r.contains("numero di desktop") && r.contains("2 contro 1"), "{r}");
        assert!(reason_for(&[attempt("a", true, None)], 1, Some("crash")).unwrap().contains("crash"), "il crash ha la precedenza");
    }

    #[test]
    fn id_lists_compare_case_insensitively_and_in_order() {
        let a = vec!["{AB}".to_string(), "{CD}".to_string()];
        assert!(same_ids(&a, &["{ab}".to_string(), "{cd}".to_string()]));
        assert!(!same_ids(&a, &["{CD}".to_string(), "{AB}".to_string()]), "l'ordine conta");
        assert!(!same_ids(&a, &["{AB}".to_string()]));
        assert!(same_ids(&[], &[]));
    }

    #[test]
    fn the_report_serializes_without_user_data() {
        let r = Report {
            build: 26300,
            revision: 9818,
            display_version: "26H2".into(),
            profiles_loaded: 1,
            user_profiles: "assente".into(),
            problems: vec![],
            attempts: vec![attempt("w11", true, None)],
            selected: Some("w11".into()),
            reason: None,
            caps: caps_for(true),
        };
        let v = serde_json::to_value(&r).unwrap();
        assert_eq!(v["build"], 26300);
        assert_eq!(v["selected"], "w11");
        assert_eq!(v["caps"]["internal"], true);
        let text = v.to_string();
        // Solo dati tecnici: nessun nome di desktop, finestra o percorso dell'utente.
        for forbidden in ["C:\\", "Users", "Raffaele", "title"] {
            assert!(!text.contains(forbidden), "{forbidden}");
        }
    }

    /// Sul sistema vero, in SOLA LETTURA: provare i profili e stampare il rapporto. Non modifica nulla.
    #[test]
    fn real_selftest_report() {
        let r = std::thread::spawn(|| report(true)).join().unwrap();
        println!("build {}.{} ({})", r.build, r.revision, r.display_version);
        println!("profili caricati: {}, selezionato: {:?}", r.profiles_loaded, r.selected);
        for a in &r.attempts {
            println!("profilo {} (dichiarato: {}): {}", a.profile, a.declared, if a.passed { "SUPERATO" } else { "non superato" });
            for c in &a.checks {
                println!("   [{}] {}: {}", if c.ok { "ok" } else { "NO" }, c.name, c.detail);
            }
        }
        println!("motivo: {:?}", r.reason);
        assert!(r.build >= 10000, "build letta");
        assert_eq!(r.caps.internal, r.selected.is_some());
        assert!(!r.attempts.is_empty() || r.reason.is_some());
    }
}
