//! User-facing dock text in Spanish, English and French. The language follows
//! the Windows display language unless the user overrides it in Settings.
//! `&` marks the menu access key.

use std::sync::atomic::{AtomicU8, Ordering};

use muelle_config::Language;
use muelle_shell::system::{UiLanguage, ui_language};

pub struct Strings {
    pub open: &'static str,
    pub new_window: &'static str,
    pub keep_in_dock: &'static str,
    pub show_in_explorer: &'static str,
    pub quit_app: &'static str,
    pub end_task: &'static str,
    pub end_task_failed: &'static str,
    pub empty_bin: &'static str,
    pub turn_hiding_on: &'static str,
    pub turn_hiding_off: &'static str,
    pub quit_dock: &'static str,
    pub start: &'static str,
    pub remove: &'static str,
    pub close_window: &'static str,
    pub ungroup: &'static str,
    pub settings: &'static str,
    pub run_as_admin: &'static str,
    pub show_desktop: &'static str,
    pub task_manager: &'static str,
    pub taskbar_settings: &'static str,
    pub usage_title: &'static str,
    pub usage_refresh: &'static str,
    pub usage_refreshing: &'static str,
    pub usage_sign_in: &'static str,
    pub usage_unavailable: &'static str,
    pub usage_loading: &'static str,
    pub usage_stale: &'static str,
    pub usage_resets: &'static str,
    pub usage_updated: &'static str,
    pub usage_used: &'static str,
    pub media_title: &'static str,
    pub media_volume: &'static str,
    pub media_no_output: &'static str,
    pub folder_empty: &'static str,
    pub folder_unavailable: &'static str,
    pub change_folder: &'static str,
    pub remove_from_dock: &'static str,
    pub shell_operation_failed: &'static str,
    pub keyboard_settings: &'static str,
    pub keyboard: &'static str,
}

const ES: Strings = Strings {
    keyboard_settings: "Ajustes de idioma y teclado…",
    keyboard: "Teclado",
    shell_operation_failed: "No se pudo iniciar la operación. Inténtalo de nuevo en un momento.",
    open: "&Abrir",
    new_window: "&Nueva ventana",
    keep_in_dock: "&Mantener en el Dock",
    show_in_explorer: "Mostrar en el &Explorador",
    quit_app: "&Salir",
    end_task: "&Finalizar tarea",
    end_task_failed: "No se pudo finalizar la tarea. La app puede haber cambiado o requerir más permisos. Vuelve a abrir su menú o usa el Administrador de tareas (Ctrl+Mayús+Esc).",
    empty_bin: "&Vaciar papelera",
    turn_hiding_on: "Activar &ocultación automática",
    turn_hiding_off: "Desactivar &ocultación automática",
    quit_dock: "Salir de &Muelle",
    start: "Inicio",
    remove: "Quitar",
    close_window: "&Cerrar",
    ungroup: "&Desagrupar",
    settings: "&Ajustes de Muelle…",
    run_as_admin: "Ejecutar como a&dministrador",
    show_desktop: "Mostrar &escritorio",
    task_manager: "Administrador de &tareas",
    taskbar_settings: "Configuración de la barra de &tareas",
    usage_title: "Uso de IA",
    usage_refresh: "Actualizar",
    usage_refreshing: "Actualizando…",
    usage_sign_in: "Inicia sesión en el cliente",
    usage_unavailable: "No disponible",
    usage_loading: "Cargando…",
    usage_stale: "Sin actualizar",
    usage_resets: "Reinicia",
    usage_updated: "Actualizado hace",
    usage_used: "usado",
    media_title: "Reproducción",
    media_volume: "Volumen del sistema",
    media_no_output: "Sin dispositivo de salida",
    folder_empty: "(vacía)",
    folder_unavailable: "Carpeta no disponible",
    change_folder: "&Cambiar de carpeta…",
    remove_from_dock: "&Quitar del Dock",
};

const EN: Strings = Strings {
    keyboard_settings: "Language and keyboard settings…",
    keyboard: "Keyboard",
    shell_operation_failed: "The operation could not start. Please try again in a moment.",
    open: "&Open",
    new_window: "&New Window",
    keep_in_dock: "&Keep in Dock",
    show_in_explorer: "Show in &Explorer",
    quit_app: "&Quit",
    end_task: "&End task",
    end_task_failed: "The task could not be ended. The app may have changed or need more permissions. Reopen its menu or use Task Manager (Ctrl+Shift+Esc).",
    empty_bin: "&Empty Recycle Bin",
    turn_hiding_on: "Turn Hiding &On",
    turn_hiding_off: "Turn Hiding &Off",
    quit_dock: "Quit &Muelle",
    start: "Start",
    remove: "Remove",
    close_window: "&Close",
    ungroup: "&Ungroup",
    settings: "Muelle &Settings…",
    run_as_admin: "Run as a&dministrator",
    show_desktop: "Show &Desktop",
    task_manager: "Task &Manager",
    taskbar_settings: "&Taskbar settings",
    usage_title: "AI usage",
    usage_refresh: "Refresh",
    usage_refreshing: "Refreshing…",
    usage_sign_in: "Sign in to the client",
    usage_unavailable: "Unavailable",
    usage_loading: "Loading…",
    usage_stale: "Not updated",
    usage_resets: "Resets",
    usage_updated: "Updated",
    usage_used: "used",
    media_title: "Now playing",
    media_volume: "System volume",
    media_no_output: "No output device",
    folder_empty: "(empty)",
    folder_unavailable: "Folder unavailable",
    change_folder: "&Change Folder…",
    remove_from_dock: "&Remove from Dock",
};

const FR: Strings = Strings {
    keyboard_settings: "Paramètres de langue et de clavier…",
    keyboard: "Clavier",
    shell_operation_failed: "L'opération n'a pas pu démarrer. Réessayez dans un instant.",
    open: "&Ouvrir",
    new_window: "&Nouvelle fenêtre",
    keep_in_dock: "&Garder dans le Dock",
    show_in_explorer: "Afficher dans l'&Explorateur",
    quit_app: "&Quitter",
    end_task: "&Forcer l’arrêt",
    end_task_failed: "Impossible de terminer la tâche. L’application a pu changer ou nécessiter plus de droits. Rouvrez son menu ou utilisez le Gestionnaire des tâches (Ctrl+Maj+Échap).",
    empty_bin: "&Vider la Corbeille",
    turn_hiding_on: "Activer le &masquage automatique",
    turn_hiding_off: "Désactiver le &masquage automatique",
    quit_dock: "&Quitter Muelle",
    start: "Démarrer",
    remove: "Retirer",
    close_window: "&Fermer",
    ungroup: "&Dégrouper",
    settings: "&Réglages de Muelle…",
    run_as_admin: "Exécuter en tant qu'&administrateur",
    show_desktop: "Afficher le &Bureau",
    task_manager: "&Gestionnaire des tâches",
    taskbar_settings: "Paramètres de la barre des &tâches",
    usage_title: "Utilisation de l'IA",
    usage_refresh: "Actualiser",
    usage_refreshing: "Actualisation…",
    usage_sign_in: "Se connecter dans le client",
    usage_unavailable: "Indisponible",
    usage_loading: "Chargement…",
    usage_stale: "Non actualisé",
    usage_resets: "Remise à zéro le",
    usage_updated: "Actualisé il y a",
    usage_used: "utilisé",
    media_title: "En cours de lecture",
    media_volume: "Volume du système",
    media_no_output: "Aucun périphérique de sortie",
    folder_empty: "(vide)",
    folder_unavailable: "Dossier introuvable",
    change_folder: "&Changer de dossier…",
    remove_from_dock: "&Retirer du Dock",
};

/// 0 = not chosen yet (follow Windows), 1 = Spanish, 2 = English, 3 = French.
static LANGUAGE: AtomicU8 = AtomicU8::new(0);

pub fn set_language(language: Language) {
    let chosen = match language {
        Language::Auto => ui_language(),
        Language::Es => UiLanguage::Spanish,
        Language::En => UiLanguage::English,
        Language::Fr => UiLanguage::French,
    };
    let stored = match chosen {
        UiLanguage::Spanish => 1,
        UiLanguage::English => 2,
        UiLanguage::French => 3,
    };
    LANGUAGE.store(stored, Ordering::Relaxed);
}

fn chosen_language() -> u8 {
    let language = LANGUAGE.load(Ordering::Relaxed);
    if language == 0 {
        set_language(Language::Auto);
        LANGUAGE.load(Ordering::Relaxed)
    } else {
        language
    }
}

pub fn text() -> &'static Strings {
    match chosen_language() {
        1 => &ES,
        3 => &FR,
        _ => &EN,
    }
}

/// The language the text is in, never `Auto`. Part of the menu bar's
/// rendered-content key.
pub fn language() -> Language {
    match chosen_language() {
        1 => Language::Es,
        3 => Language::Fr,
        _ => Language::En,
    }
}
