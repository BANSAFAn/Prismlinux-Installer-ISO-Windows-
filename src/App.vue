<script setup lang="ts">
import { ref, onMounted, onUnmounted, computed } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { listen, UnlistenFn } from "@tauri-apps/api/event";

interface UsbDrive {
  Number: number;
  FriendlyName: string;
  Size: number;
}

interface ProgressEvent {
  downloaded?: number;
  written?: number;
  total: number;
  speed: number;
}

const currentScreen = ref<
  | "dashboard"
  | "flasher"
  | "iso-selector"
  | "hdd-installer"
  | "progress-download"
  | "progress-flash"
  | "progress-hdd"
  | "success-flash"
  | "success-hdd"
>("dashboard");

const lang = ref<"ua" | "en" | "de" | "es" | "ar" | "zh" | "ja" | "hi">("ua");
const selectedMode = ref<"flash" | "hdd" | null>(null);
const isoSource = ref<"download" | "local">("download");
const localIsoPath = ref<string>("");
const errorMessage = ref<string>("");
const showExitModal = ref(false);
const isLangDropdownOpen = ref(false);
const isLinux = ref(false);

const usbDrives = ref<UsbDrive[]>([]);
const selectedDriveNum = ref<number | null>(null);
const isRefreshingDrives = ref(false);

const confirmSafety1 = ref(false);
const confirmSafety2 = ref(false);
const confirmHdd1 = ref(false);
const confirmHdd2 = ref(false);

const progressBytes = ref(0);
const progressTotal = ref(0);
const progressSpeed = ref(0);
const hddStatusText = ref("");

let unlistenDownload: UnlistenFn | null = null;
let unlistenFlash: UnlistenFn | null = null;
let unlistenClose: UnlistenFn | null = null;

const languagesList = [
  { code: "ua", name: "Українська" },
  { code: "en", name: "English" },
  { code: "de", name: "Deutsch" },
  { code: "es", name: "Español" },
  { code: "ar", name: "العربية" },
  { code: "zh", name: "简体中文" },
  { code: "ja", name: "日本語" },
  { code: "hi", name: "हिन्दी" }
];

const translations = {
  ua: {
    title: "PRISM LINUX INSTALLER",
    dashboard_subtitle: "Оберіть спосіб розгортання операційної системи",
    card_flash_title: "Записати на флешку",
    card_flash_desc: "Створіть завантажувальний USB-накопичувач для встановлення Prism Linux на будь-який комп'ютер. Це найбезпечніший і рекомендований спосіб.",
    card_hdd_title: "Встановити на цей ПК",
    card_hdd_desc: "Встановіть систему без флешки! Ми створимо завантажувальний розділ на вашому диску та додамо Prism Linux до меню запуску комп'ютера.",
    btn_back: "Назад",
    btn_cancel: "Скасувати",
    btn_next: "Далі",
    iso_title: "Оберіть джерело образу",
    iso_download: "Завантажити останній образ з мережі",
    iso_download_desc: "Рекомендовано. Ми автоматично завантажимо найновішу, стабільну версію Prism Linux з офіційних серверів.",
    iso_local: "Використати локальний файл (.iso)",
    iso_local_desc: "Оберіть вже завантажений образ диска з вашого комп'ютера.",
    iso_select_file: "Оберіть файл образу",
    iso_selected_path: "Обрано образ: ",
    btn_start_download: "Розпочати запис",
    downloading_title: "Завантаження образу диска",
    speed: "Швидкість: ",
    eta: "Залишилось часу: ",
    bytes_downloaded: "Завантажено: ",
    flasher_title: "Запис образу на USB",
    flasher_subtitle: "Налаштування створення завантажувальної флешки Prism Linux",
    select_drive: "Оберіть USB-накопичувач:",
    no_drives_found: "USB-накопичувачі не знайдені. Вставте флешку та оновіть список.",
    btn_refresh: "Оновити список",
    safety_title: "КРИТИЧНО ВАЖЛИВА БЕЗПЕКА",
    safety_desc: "Запис образу призведе до повного знищення всіх файлів на обраному пристрої! Будь ласка, перевірте, чи не вибрано ваш основний диск.",
    confirm_safety_1: "Я розумію, що всі файли на обраному накопичувачі будуть безповоротно видалені.",
    confirm_safety_2: "Я підтверджую, що вибрав саме USB-флешку, а не свій системний жорсткий диск.",
    btn_flash: "Розпочати запис флешки",
    flashing_title: "Запис образу на флешку...",
    flashing_desc: "Будь ласка, не виймайте флешку та не вимикайте комп'ютер. Цей процес може зайняти кілька хвилин.",
    bytes_written: "Записано: ",
    success_flash_title: "Флешку успішно записано!",
    success_flash_desc: "Ваш завантажувальний накопичувач Prism Linux повністю готовий. Перезавантажте комп'ютер, увійдіть в BIOS/Boot menu та оберіть вашу флешку.",
    btn_done: "Завершити",
    hdd_title: "Пряме встановлення на пристрій",
    hdd_subtitle: "Налаштування запуску інсталятора з вашого жорсткого диска",
    hdd_warning_title: "Увага: Обмеження шифрування та BitLocker",
    hdd_warning_desc: "Якщо ваш диск C: зашифровано через BitLocker (стандартно на більшості нових ноутбуків), інсталятор Linux не зможе прочитати файли під час запуску. Переконайтеся, що BitLocker тимчасово вимкнено або призупинено в Windows перед перезавантаженням!",
    hdd_step_1_title: "Копіювання файлів встановлення",
    hdd_step_1_desc: "Ми створимо директорію C:\\PrismLinux і скопіюємо туди ядро та образ.",
    hdd_step_2_title: "Встановлення автономного завантажувача",
    hdd_step_2_desc: "Ми встановимо безпечний, підписаний Microsoft завантажувач GRUB у ваш EFI-розділ.",
    hdd_step_3_title: "Додавання пункту в Boot Manager Windows",
    hdd_step_3_desc: "При запуску ПК з'явиться меню вибору: запустити Windows або розпочати встановлення Prism Linux.",
    flash_step_1_title: "Підготовка образу диска",
    flash_step_1_desc: "Ми завантажимо найновіший образ Prism Linux або підготуємо обраний вами локальний файл.",
    flash_step_2_title: "Блокування USB-накопичувача",
    flash_step_2_desc: "Ми заблокуємо обрану флешку та розмонтуємо її розділи для безпечного посекторного запису.",
    flash_step_3_title: "Посекторний запис даних",
    flash_step_3_desc: "Образ буде записано на флешку block-by-block, після чого пристрій буде розблоковано та готово до завантаження.",
    confirm_hdd_1: "Я підтверджую, що мій диск C: не зашифрований BitLocker (або шифрування призупинено).",
    confirm_hdd_2: "Я розумію, що додаю пункт встановлення нової ОС до меню завантаження мого ПК.",
    btn_install_hdd: "Налаштувати прямий запуск",
    hdd_status_working: "Налаштування завантажувача... Будь ласка, зачекайте.",
    success_hdd_title: "Систему успішно налаштовано!",
    success_hdd_desc: "Всі необхідні файли скопійовано, а завантажувач налаштовано. Перезавантажте комп'ютер прямо зараз. У меню Windows Boot Manager оберіть пункт 'Prism Linux Live Installer', щоб розпочати встановлення системи без будь-яких флешок!",
    btn_reboot_now: "Перезавантажити зараз",
    error_title: "Сталася помилка",
    btn_ok: "Зрозуміло",
    exit_title: "Вихід з програми",
    exit_confirm_msg: "Ви впевнені, що хочете зупинити процес та вийти з інсталятора?",
    btn_yes: "Так, вийти",
    btn_no: "Ні, залишитися",
  },
  en: {
    title: "PRISM LINUX INSTALLER",
    dashboard_subtitle: "Select operating system deployment method",
    card_flash_title: "Write to USB Flash Drive",
    card_flash_desc: "Create a bootable USB drive to install Prism Linux on any computer. This is the safest and highly recommended method.",
    card_hdd_title: "Install on this PC",
    card_hdd_desc: "Install without a USB drive! We will create a local boot partition on your drive and add Prism Linux to your computer's boot menu.",
    btn_back: "Back",
    btn_cancel: "Cancel",
    btn_next: "Next",
    iso_title: "Select ISO Source",
    iso_download: "Download the latest image online",
    iso_download_desc: "Recommended. We will automatically download the newest stable release of Prism Linux from official servers.",
    iso_local: "Use a local file (.iso)",
    iso_local_desc: "Select a previously downloaded disk image file from your computer.",
    iso_select_file: "Select Image File",
    iso_selected_path: "Selected image: ",
    btn_start_download: "Start Flashing",
    downloading_title: "Downloading Disk Image",
    speed: "Speed: ",
    eta: "Remaining time: ",
    bytes_downloaded: "Downloaded: ",
    flasher_title: "USB Flashing Setup",
    flasher_subtitle: "Configuring Prism Linux bootable USB creation",
    select_drive: "Select USB Drive:",
    no_drives_found: "No USB drives detected. Insert a flash drive and refresh the list.",
    btn_refresh: "Refresh list",
    safety_title: "CRITICAL SAFETY INFORMATION",
    safety_desc: "Flashing the image will completely destroy all files on the selected device! Please double check that you haven't selected your system partition.",
    confirm_safety_1: "I understand that all files on the selected drive will be permanently deleted.",
    confirm_safety_2: "I confirm that I have selected a USB flash drive, not my primary OS hard disk.",
    btn_flash: "Start USB Flashing",
    flashing_title: "Writing image to USB...",
    flashing_desc: "Please do not eject the drive or turn off your computer. This process may take several minutes.",
    bytes_written: "Written: ",
    success_flash_title: "USB Drive Successfully Written!",
    success_flash_desc: "Your Prism Linux bootable drive is fully ready. Restart your computer, enter BIOS/Boot menu and select your USB drive.",
    btn_done: "Done",
    hdd_title: "Direct Device Installation",
    hdd_subtitle: "Configuring installer startup directly from your hard drive",
    hdd_warning_title: "Warning: Encryption & BitLocker Limits",
    hdd_warning_desc: "If your C: drive is encrypted with BitLocker (default on most new laptops), the Linux installer will be unable to read the files during boot. Ensure that BitLocker is temporarily disabled or suspended in Windows before rebooting!",
    hdd_step_1_title: "Staging installation files",
    hdd_step_1_desc: "We will create a C:\\PrismLinux folder and copy the kernel and ISO image there.",
    hdd_step_2_title: "Installing standalone bootloader",
    hdd_step_2_desc: "We will install a secure, Microsoft-signed GRUB bootloader into your EFI system partition.",
    hdd_step_3_title: "Adding Windows Boot Manager option",
    hdd_step_3_desc: "Upon booting, you will be presented with a menu choice: start Windows or launch Prism Linux Installer.",
    flash_step_1_title: "Preparing disk image",
    flash_step_1_desc: "We will download the latest Prism Linux image or stage your selected local file.",
    flash_step_2_title: "Locking USB drive",
    flash_step_2_desc: "We will exclusively lock the selected USB drive and dismount its partitions for safe block writing.",
    flash_step_3_title: "Block-by-block writing",
    flash_step_3_desc: "The image will be written block-by-block, then the drive will be unlocked and ready for booting.",
    confirm_hdd_1: "I confirm that my C: drive is not encrypted with BitLocker (or encryption is suspended).",
    confirm_hdd_2: "I understand that I am adding a new boot option to my PC startup menu.",
    btn_install_hdd: "Configure Direct Boot",
    hdd_status_working: "Configuring bootloader... Please wait.",
    success_hdd_title: "System Configured Successfully!",
    success_hdd_desc: "All necessary files have been copied and the bootloader has been set up. Restart your computer now. In the Windows Boot Manager menu, select 'Prism Linux Live Installer' to install without any USB drives!",
    btn_reboot_now: "Reboot Now",
    error_title: "An Error Occurred",
    btn_ok: "Got it",
    exit_title: "Exit Installer",
    exit_confirm_msg: "Are you sure you want to stop the process and exit the installer?",
    btn_yes: "Yes, exit",
    btn_no: "No, stay",
  },
  de: {
    title: "PRISM LINUX INSTALLER",
    dashboard_subtitle: "Wählen Sie die Bereitstellungsmethode des Betriebssystems",
    card_flash_title: "Auf USB-Stick schreiben",
    card_flash_desc: "Erstellen Sie ein bootfähiges USB-Laufwerk für Prism Linux. Dies ist die sicherste und empfohlene Methode.",
    card_hdd_title: "Auf diesem PC installieren",
    card_hdd_desc: "Installation ohne USB-Stick! Wir erstellen eine Partition und fügen Prism Linux zum Bootmenü hinzu.",
    btn_back: "Zurück",
    btn_cancel: "Abbrechen",
    btn_next: "Weiter",
    iso_title: "ISO-Quelle auswählen",
    iso_download: "Aktuelles Image online herunterladen",
    iso_download_desc: "Empfohlen. Automatischer Download des stabilen Images von den Prism Linux Servern.",
    iso_local: "Lokale Datei (.iso) verwenden",
    iso_local_desc: "Wählen Sie eine bereits heruntergeladene Image-Datei von Ihrem Computer aus.",
    iso_select_file: "Image-Datei auswählen",
    iso_selected_path: "Ausgewähltes Image: ",
    btn_start_download: "Starten",
    downloading_title: "Herunterladen des Images",
    speed: "Geschwindigkeit: ",
    eta: "Verbleibende Zeit: ",
    bytes_downloaded: "Heruntergeladen: ",
    flasher_title: "USB-Flash-Einrichtung",
    flasher_subtitle: "Konfiguration für das Erstellen des Prism Linux USB-Laufwerks",
    select_drive: "USB-Laufwerk auswählen:",
    no_drives_found: "Keine USB-Laufwerke erkannt. Bitte stecken Sie einen USB-Stick ein.",
    btn_refresh: "Liste aktualisieren",
    safety_title: "SICHERHEITSHINWEIS",
    safety_desc: "Der Schreibvorgang löscht alle vorhandenen Daten auf dem ausgewählten Laufwerk unwiderruflich!",
    confirm_safety_1: "Ich verstehe, dass alle Dateien auf der флешці gelöscht werden.",
    confirm_safety_2: "Ich bestätige, dass ich ein USB-Laufwerk ausgewählt habe.",
    btn_flash: "USB-Flash starten",
    flashing_title: "Schreibe Image...",
    flashing_desc: "Bitte entfernen Sie das Laufwerk nicht und schalten Sie den PC nicht aus.",
    bytes_written: "Geschrieben: ",
    success_flash_title: "USB-Stick bereit!",
    success_flash_desc: "Ihr Prism Linux-Laufwerk ist bereit. Starten Sie neu und wählen Sie es im BIOS-Bootmenü aus.",
    btn_done: "Fertig",
    hdd_title: "Direkte Festplatten-Installation",
    hdd_subtitle: "Starten des Installers direkt von der Festplatte einrichten",
    hdd_warning_title: "Warnung: BitLocker-Verschlüsselung",
    hdd_warning_desc: "Wenn C: mit BitLocker verschlüsselt ist, kann der Linux-Installer beim Booten nicht auf die Dateien zugreifen. Deaktivieren Sie BitLocker vor dem Neustart!",
    hdd_step_1_title: "Dateien kopieren",
    hdd_step_1_desc: "Wir kopieren das Image und den Kernel nach C:\\PrismLinux.",
    hdd_step_2_title: "Bootloader einrichten",
    hdd_step_2_desc: "Wir installieren den GRUB-Bootloader in Ihrer EFI-Systempartition.",
    hdd_step_3_title: "Starteintrag hinzufügen",
    hdd_step_3_desc: "Wir fügen den Starteintrag zum Windows Boot Manager hinzu.",
    flash_step_1_title: "Image vorbereiten",
    flash_step_1_desc: "Wir laden das Image herunter oder bereiten die lokale Datei vor.",
    flash_step_2_title: "Laufwerk sperren",
    flash_step_2_desc: "Wir sperren das USB-Laufwerk für den exklusiven Schreibzugriff.",
    flash_step_3_title: "Blockweiser Schreibvorgang",
    flash_step_3_desc: "Der blockweiser Schreibvorgang wird gestartet und verifiziert.",
    confirm_hdd_1: "Ich bestätige, dass mein Laufwerk C: nicht mit BitLocker verschlüsselt ist.",
    confirm_hdd_2: "Ich stimme dem Hinzufügen des Eintrags zum Bootmenü zu.",
    btn_install_hdd: "Direktstart einrichten",
    hdd_status_working: "Richte Bootloader ein... Bitte warten.",
    success_hdd_title: "System bereit!",
    success_hdd_desc: "Dateien wurden kopiert. Starten Sie neu und wählen Sie 'Prism Linux Live Installer' im Bootmenü.",
    btn_reboot_now: "Jetzt neu starten",
    error_title: "Fehler aufgetreten",
    btn_ok: "Verstanden",
    exit_title: "Beenden",
    exit_confirm_msg: "Sind Sie sicher, dass Sie den Vorgang abbrechen und den Installer schließen möchten?",
    btn_yes: "Ja, beenden",
    btn_no: "Nein, bleiben",
  },
  es: {
    title: "PRISM LINUX INSTALLER",
    dashboard_subtitle: "Seleccione el método de instalación del sistema operativo",
    card_flash_title: "Grabar en USB",
    card_flash_desc: "Cree una unidad USB de arranque para instalar Prism Linux en cualquier PC. Este es el método más seguro y recomendado.",
    card_hdd_title: "Instalar en esta PC",
    card_hdd_desc: "¡Instale sin unidad USB! Crearemos una partición local de arranque y agregaremos la opción al menú de inicio.",
    btn_back: "Atrás",
    btn_cancel: "Cancelar",
    btn_next: "Siguiente",
    iso_title: "Seleccionar origen ISO",
    iso_download: "Descargar la imagen más reciente en línea",
    iso_download_desc: "Recomendado. Descarga automática de la última versión estable desde los servidores oficiales.",
    iso_local: "Usar un archivo local (.iso)",
    iso_local_desc: "Seleccione un archivo de imagen previamente descargado de su computadora.",
    iso_select_file: "Seleccionar archivo de imagen",
    iso_selected_path: "Imagen seleccionada: ",
    btn_start_download: "Iniciar",
    downloading_title: "Descargando imagen",
    speed: "Velocidad: ",
    eta: "Tiempo restante: ",
    bytes_downloaded: "Descargado: ",
    flasher_title: "Configuración de USB",
    flasher_subtitle: "Configuración para la creación de la unidad USB de Prism Linux",
    select_drive: "Seleccionar unidad USB:",
    no_drives_found: "No se detectaron unidades USB. Inserte una unidad y actualice la lista.",
    btn_refresh: "Actualizar lista",
    safety_title: "INFORMACIÓN DE SEGURIDAD CRÍTICA",
    safety_desc: "¡La grabación de la imagen destruirá permanentemente todos los datos de la unidad seleccionada!",
    confirm_safety_1: "Entiendo que todos los archivos en la unidad seleccionada se eliminarán.",
    confirm_safety_2: "Confirmo que he seleccionado una unidad USB de almacenamiento.",
    btn_flash: "Iniciar grabación USB",
    flashing_title: "Grabando imagen...",
    flashing_desc: "No extraiga la unidad ni apague la computadora durante el proceso.",
    bytes_written: "Escrito: ",
    success_flash_title: "¡Unidad USB lista!",
    success_flash_desc: "La unidad de arranque de Prism Linux está lista. Reinicie su PC y selecciónela en el menú de arranque del BIOS.",
    btn_done: "Terminar",
    hdd_title: "Instalación directa en disco",
    hdd_subtitle: "Configurar el inicio del instalador directamente desde su disco duro",
    hdd_warning_title: "Advertencia: Cifrado y BitLocker",
    hdd_warning_desc: "Si su unidad C: está cifrada con BitLocker, el instalador no podrá leer los archivos al iniciar. ¡Suspenda BitLocker antes de reiniciar!",
    hdd_step_1_title: "Copiar archivos",
    hdd_step_1_desc: "Crearemos la carpeta C:\\PrismLinux y copiaremos el núcleo y la imagen allí.",
    hdd_step_2_title: "Instalar cargador de arranque",
    hdd_step_2_desc: "Instalaremos el cargador de arranque GRUB firmado en su partición del sistema EFI.",
    hdd_step_3_title: "Agregar opción de arranque de Windows",
    hdd_step_3_desc: "Se agregará la opción al menú de inicio de Windows Boot Manager.",
    flash_step_1_title: "Preparar imagen",
    flash_step_1_desc: "Descargaremos la última imagen o prepararemos el archivo local seleccionado.",
    flash_step_2_title: "Bloquear unidad",
    flash_step_2_desc: "Bloquearemos la unidad USB de forma exclusiva para garantizar un proceso seguro.",
    flash_step_3_title: "Grabación por bloques",
    flash_step_3_desc: "Se realizará la copia por bloques de datos a la unidad USB física.",
    confirm_hdd_1: "Confirmo que mi unidad C: no está cifrada con BitLocker.",
    confirm_hdd_2: "Acepto agregar la opción al menú de arranque de mi computadora.",
    btn_install_hdd: "Configurar inicio directo",
    hdd_status_working: "Configurando cargador de arranque... Espere.",
    success_hdd_title: "¡Sistema configurado!",
    success_hdd_desc: "Configuración exitosa. Reinicie y elija 'Prism Linux Live Installer' en el menú de inicio.",
    btn_reboot_now: "Reiniciar ahora",
    error_title: "Ha ocurrido un error",
    btn_ok: "Entendido",
    exit_title: "Salir",
    exit_confirm_msg: "¿Está seguro de que desea detener el proceso y salir del instalador?",
    btn_yes: "Sí, salir",
    btn_no: "No, quedarse",
  },
  ar: {
    title: "مساعد تثبيت بريزم لينكس",
    dashboard_subtitle: "اختر طريقة نشر نظام التشغيل",
    card_flash_title: "الكتابة على ذاكرة فلاش",
    card_flash_desc: "أنشئ محرك أقراص USB قابل للإقلاع لتثبيت بريزم لينكس على أي جهاز. هذه هي الطريقة الأكثر أماناً وموثوقية.",
    card_hdd_title: "التثبيت المباشر على هذا الجهاز",
    card_hdd_desc: "تثبيت بدون الحاجة لذاكرة فلاش! سنقوم بتهيئة قسم إقلاع محلي وإضافة الخيار إلى قائمة بدء التشغيل الخاصة بالكمبيوتر.",
    btn_back: "رجوع",
    btn_cancel: "إلغاء",
    btn_next: "التالي",
    iso_title: "اختر مصدر ملف الـ ISO",
    iso_download: "تنزيل أحدث صورة مباشرة من الإنترنت",
    iso_download_desc: "موصى به. سيتم تلقائياً تنزيل أحدث إصدار مستقر من بريزم لينكس من الخوادم الرسمية.",
    iso_local: "استخدام ملف محلي (.iso)",
    iso_local_desc: "اختر صورة قرص تم تنزيلها مسبقاً على جهاز الكمبيوتر الخاص بك.",
    iso_select_file: "اختر ملف الصورة",
    iso_selected_path: "الصورة المحددة: ",
    btn_start_download: "بدء التثبيت",
    downloading_title: "جاري تنزيل صورة القرص",
    speed: "السرعة: ",
    eta: "الوقت المتبقي: ",
    bytes_downloaded: "تم تنزيل: ",
    flasher_title: "تهيئة الذاكرة الوميضية",
    flasher_subtitle: "إعداد وتجهيز ذاكرة الـ USB الخاصة بنظام بريزم لينكس",
    select_drive: "اختر محرك أقراص الـ USB:",
    no_drives_found: "لم يتم العثور على محركات أقراص USB. يرجى إدخال فلاشة وتحديث القائمة.",
    btn_refresh: "تحديث القائمة",
    safety_title: "تحذير أمان هام للغاية",
    safety_desc: "تنبيه: ستؤدي عملية الكتابة إلى مسح جميع البيانات والملفات الموجودة على محرك الأقراص المحدد نهائياً!",
    confirm_safety_1: "أفهم تماماً أنه سيتم حذف كافة الملفات على الفلاشة بشكل نهائي.",
    confirm_safety_2: "أؤكد أنني قمت باختيار محرك أقراص USB وليس القرص الصلب للنظام.",
    btn_flash: "بدء عملية الكتابة على الـ USB",
    flashing_title: "جاري كتابة الصورة على الفلاشة...",
    flashing_desc: "يرجى عدم إزالة الفلاشة أو إطفاء الكمبيوتر أثناء الكتابة لتفادي تلف البيانات.",
    bytes_written: "المكتوب: ",
    success_flash_title: "تمت كتابة الصورة بنجاح!",
    success_flash_desc: "محرك أقراص بريزم لينكس أصبح جاهزاً. أعد تشغيل الكمبيوتر، وادخل إلى واجهة الـ BIOS واختر الإقلاع من الـ USB.",
    btn_done: "إنهاء",
    hdd_title: "التثبيت المباشر من القرص الصلب",
    hdd_subtitle: "إعداد مشغل التثبيت للعمل مباشرة من القرص الصلب الخاص بك",
    hdd_warning_title: "تنبيه: تشفير القرص عبر الـ BitLocker",
    hdd_warning_desc: "إذا كان القرص C: مشفراً باستخدام BitLocker، فلن يتمكن نظام التثبيت من قراءة الملفات عند الإقلاع. يرجى إيقاف تشفير BitLocker مؤقتاً!",
    hdd_step_1_title: "نسخ ملفات التثبيت",
    hdd_step_1_desc: "سنقوم بإنشاء المجلد C:\\PrismLinux ونسخ النواة والملفات الأساسية إليه.",
    hdd_step_2_title: "تثبيت محمل إقلاع مستقل",
    hdd_step_2_desc: "سنقوم بتثبيت محمل الإقلاع المعتمد GRUB داخل قسم النظام الخاص بالـ EFI.",
    hdd_step_3_title: "إضافة خيار لقائمة الإقلاع بويندوز",
    hdd_step_3_desc: "ستظهر قائمة اختيار نظام التشغيل عند بدء تشغيل الكمبيوتر لتحديد نظام التثبيت.",
    flash_step_1_title: "تحضير صورة القرص",
    flash_step_1_desc: "سنقوم بتحميل الصورة أو تجهيز الملف المحلي المحدد.",
    flash_step_2_title: "قفل محرك الأقراص",
    flash_step_2_desc: "سنقوم بإقفال الفلاشة وحظر الوصول إليها من الأنظمة الأخرى لضمان كتابة آمنة.",
    flash_step_3_title: "كتابة البيانات بالكتل",
    flash_step_3_desc: "جاري نسخ البيانات block-by-block إلى ذاكرة الفلاش المحددة.",
    confirm_hdd_1: "أؤكد أن القرص C: الخاص بي غير مشفر بواسطة BitLocker.",
    confirm_hdd_2: "أوافق على إضافة خيار التثبيت إلى قائمة إقلاع الكمبيوتر.",
    btn_install_hdd: "إعداد الإقلاع المباشر",
    hdd_status_working: "جاري تهيئة محمل الإقلاع... يرجى الانتظار.",
    success_hdd_title: "اكتمل الإعداد بنجاح!",
    success_hdd_desc: "تم نسخ الملفات بنجاح. أعد تشغيل الكمبيوتر الآن، واختر 'Prism Linux Live Installer' من قائمة بدء تشغيل ويندوز.",
    btn_reboot_now: "إعادة التشغيل الآن",
    error_title: "حدث خطأ ما",
    btn_ok: "حسناً، فهمت",
    exit_title: "الخروج من البرنامج",
    exit_confirm_msg: "هل أنت متأكد أنك تريد إلغاء العملية والخروج من مساعد التثبيت؟",
    btn_yes: "نعم، خروج",
    btn_no: "لا، البقاء",
  },
  zh: {
    title: "PRISM LINUX 安装程序",
    dashboard_subtitle: "选择操作系统部署方式",
    card_flash_title: "写入到 U 盘",
    card_flash_desc: "创建可启动的 U 盘以在任何电脑上安装 Prism Linux。这是最安全且推荐的方式。",
    card_hdd_title: "直接安装在此电脑",
    card_hdd_desc: "无需 U 盘安装！我们将在您的硬盘上创建一个引导分区，并将 Prism Linux 添加至启动菜单中。",
    btn_back: "返回",
    btn_cancel: "取消",
    btn_next: "下一步",
    iso_title: "选择 ISO 镜像源",
    iso_download: "在线下载最新版系统镜像",
    iso_download_desc: "推荐。我们将自动从官方服务器下载最新的稳定版 Prism Linux 镜像。",
    iso_local: "使用本地文件 (.iso)",
    iso_local_desc: "选择您电脑中已下载好的光盘镜像文件。",
    iso_select_file: "选择镜像文件",
    iso_selected_path: "已选镜像: ",
    btn_start_download: "开始写入",
    downloading_title: "正在下载系统镜像",
    speed: "速度: ",
    eta: "剩余时间: ",
    bytes_downloaded: "已下载: ",
    flasher_title: "U 盘写入设置",
    flasher_subtitle: "配置并创建 Prism Linux 可启动 U 盘",
    select_drive: "选择 U 盘设备:",
    no_drives_found: "未检测到 U 盘。请插入 U 盘并刷新列表。",
    btn_refresh: "刷新列表",
    safety_title: "关键安全提示",
    safety_desc: "写入镜像将会彻底清除所选设备上的所有数据！请仔细核对，确保未选中您的系统分区。",
    confirm_safety_1: "我明白所选 U 盘上的所有文件都将被永久删除。",
    confirm_safety_2: "我确认选择的是外部 U 盘，而非系统主硬盘。",
    btn_flash: "开始 U 盘写入",
    flashing_title: "正在写入镜像到 U 盘...",
    flashing_desc: "请勿拔出 U 盘或关闭电脑。此过程可能需要几分钟时间。",
    bytes_written: "已写入: ",
    success_flash_title: "U 盘写入成功！",
    success_flash_desc: "您的 Prism Linux 引导盘已准备就绪。重启电脑并进入 BIOS 启动菜单，选择 U 盘引导启动。",
    btn_done: "完成",
    hdd_title: "直接硬盘安装",
    hdd_subtitle: "配置并直接从您的硬盘启动安装程序",
    hdd_warning_title: "警告: BitLocker 磁盘加密限制",
    hdd_warning_desc: "若您的 C 盘已启用 BitLocker 加密，Linux 安装程序将在启动时无法读取文件。请在重启前在 Windows 中临时关闭或暂停 BitLocker！",
    hdd_step_1_title: "复制安装文件",
    hdd_step_1_desc: "我们将在 C 盘根目录创建 C:\\PrismLinux 文件夹，并将内核和镜像复制到此。",
    hdd_step_2_title: "安装独立引导加载器",
    hdd_step_2_desc: "我们将在您的 EFI 系统分区中安装安全、具有微软签名的 GRUB 引导加载器。",
    hdd_step_3_title: "添加 Windows 启动管理器菜单项",
    hdd_step_3_desc: "电脑启动时将显示选项菜单: 启动 Windows 或进入 Prism Linux 安装程序。",
    flash_step_1_title: "准备系统镜像",
    flash_step_1_desc: "我们将自动下载镜像文件或准备您选择的本地镜像。",
    flash_step_2_title: "锁定 U 盘设备",
    flash_step_2_desc: "我们将独占锁定 U 盘并卸载其分区，以确保写入过程的绝对安全。",
    flash_step_3_title: "按数据块写入",
    flash_step_3_desc: "系统镜像正按 block-by-block 写入到您指定的 U 盘中。",
    confirm_hdd_1: "我确认我的 C 盘没有启用 BitLocker 磁盘加密功能。",
    confirm_hdd_2: "我同意在电脑的系统启动菜单中添加一个全新的引导菜单项。",
    btn_install_hdd: "配置硬盘启动",
    hdd_status_working: "正在配置引导加载器... 请稍候。",
    success_hdd_title: "系统配置完成！",
    success_hdd_desc: "配置成功。立即重启电脑，并在启动菜单中选择 'Prism Linux Live Installer'。",
    btn_reboot_now: "立即重启",
    error_title: "发生错误",
    btn_ok: "知道了",
    exit_title: "退出安装程序",
    exit_confirm_msg: "您确定要中止此过程并退出安装程序吗？",
    btn_yes: "是的，退出",
    btn_no: "不，留下来",
  },
  ja: {
    title: "PRISM LINUX インストーラー",
    dashboard_subtitle: "オペレーティングシステムの配置方法を選択してください",
    card_flash_title: "USBに書き込む",
    card_flash_desc: "Prism Linuxをインストールするための起動可能なUSBドライブを作成します。これが最も安全で推奨される方法です。",
    card_hdd_title: "このPCにインストール",
    card_hdd_desc: "USBなしでインストール！ハードドライブに起動用パーティションを作成し、PC의起動メニューにPrism Linuxを追加します。",
    btn_back: "戻る",
    btn_cancel: "キャンセル",
    btn_next: "次へ",
    iso_title: "ISOイメージの選択",
    iso_download: "最新のイメージをオンラインでダウンロード",
    iso_download_desc: "推奨。公式サーバーから最新の安定版Prism Linuxイメージを自動的にダウンロードします。",
    iso_local: "ローカルファイル (.iso) を使用",
    iso_local_desc: "コンピューターにすでにダウンロードされているディスクイメージファイルを選択します。",
    iso_select_file: "イメージファイルを選択",
    iso_selected_path: "選択されたイメージ: ",
    btn_start_download: "書き込み開始",
    downloading_title: "イメージファイルをダウンロード中",
    speed: "速度: ",
    eta: "残り時間: ",
    bytes_downloaded: "ダウンロード済み: ",
    flasher_title: "USB書き込み設定",
    flasher_subtitle: "Prism Linux起動可能USB作成の構成",
    select_drive: "USBドライブを選択してください:",
    no_drives_found: "USBドライブが検出されません。ドライブを挿入してリストを更新してください。",
    btn_refresh: "リストを更新",
    safety_title: "重要なセキュリティ情報",
    safety_desc: "イメージを書き込むと、選択したデバイス上のすべてのデータが完全に削除されます！システムパーティションが選択されていないか再確認してください。",
    confirm_safety_1: "選択したドライブ上のすべてのファイルが永続的に削除されることを理解しています。",
    confirm_safety_2: "プライマリOSディスクではなく、外部USBドライブを選択したことを確認します。",
    btn_flash: "USB書き込みを開始",
    flashing_title: "USBにイメージを書き込み中...",
    flashing_desc: "ドライブを抜き取ったり、コンピューターの電源を切ったりしないでください。このプロセスには数分かかる場合があります。",
    bytes_written: "書き込み済み: ",
    success_flash_title: "USB書き込みが成功しました！",
    success_flash_desc: "Prism Linux起動ドライブの準備が完了しました。コンピューターを再起動し、BIOS起動メニューでUSBドライブを選択してください。",
    btn_done: "完了",
    hdd_title: "直接ハードドライブインストール",
    hdd_subtitle: "ハードドライブから直接インストーラーを起動するように構成します",
    hdd_warning_title: "警告: 暗号化とBitLockerの制限",
    hdd_warning_desc: "C:ドライブがBitLockerで暗号化されている場合、Linuxインストーラーは起動時にファイルを読み取ることができません。再起動前にWindowsで一時的にBitLockerをオフまたは一時停止してください！",
    hdd_step_1_title: "インストールファイルの配置",
    hdd_step_1_desc: "C:\\PrismLinuxフォルダーを作成し、そこにカーネルとイメージをコピーします。",
    hdd_step_2_title: "スタンドアロンブートローダーのインストール",
    hdd_step_2_desc: "EFIシステムパーティションに、安全でMicrosoft署名済みのGRUBブートローダーをインストールします。",
    hdd_step_3_title: "Windowsブートマネージャーへの追加",
    hdd_step_3_desc: "PCの起動時に、Windowsを起動するかPrism Linuxインストーラーを実行するかのメニューが表示されます。",
    flash_step_1_title: "イメージの準備",
    flash_step_1_desc: "最新のイメージをダウンロードするか、選択したローカルファイルを準備します。",
    flash_step_2_title: "ドライブのロック",
    flash_step_2_desc: "書き込みプロセスを安全に実行するため、USBドライブを排他的にロックします。",
    flash_step_3_title: "ブロック書き込み",
    flash_step_3_desc: "システムイメージがブロック単位で指定のUSBドライブに書き込まれます。",
    confirm_hdd_1: "C:ドライブがBitLockerで暗号化されていないことを確認します。",
    confirm_hdd_2: "PCのシステム起動メニューに新しいメニュー項目を追加することに同意します。",
    btn_install_hdd: "直接起動を構成",
    hdd_status_working: "ブートローダーを構成中... しばらくお待ちください。",
    success_hdd_title: "システム構成完了！",
    success_hdd_desc: "構成成功。再起動し、Windows起動メニューで 'Prism Linux Live Installer' を選択してください。",
    btn_reboot_now: "今すぐ再起動",
    error_title: "エラーが発生しました",
    btn_ok: "了解しました",
    exit_title: "インストーラーを終了",
    exit_confirm_msg: "プロセスを中止してインストーラーを終了してもよろしいですか？",
    btn_yes: "はい、終了します",
    btn_no: "いいえ、残ります",
  },
  hi: {
    title: "प्रिज्म लिनक्स इंस्टॉलर",
    dashboard_subtitle: "ऑपरेटिंग सिस्टम स्थापना विधि का चयन करें",
    card_flash_title: "यूएसबी फ्लैश ड्राइव पर लिखें",
    card_flash_desc: "किसी भी कंप्यूटर पर प्रिज्म लिनक्स स्थापित करने के लिए एक बूट करने योग्य यूएसबी ड्राइव बनाएं। यह सबसे सुरक्षित और अनुशंसित तरीका है।",
    card_hdd_title: "इस कंप्यूटर पर स्थापित करें",
    card_hdd_desc: "यूएसबी ड्राइव के बिना स्थापित करें! हम आपके कंप्यूटर पर प्रिज्म लिनक्स को बूट मेनू में जोड़ देंगे।",
    btn_back: "पीछे",
    btn_cancel: "रद्द करें",
    btn_next: "आगे",
    iso_title: "आईएसओ स्रोत का चयन करें",
    iso_download: "नवीनतम संस्करण ऑनलाइन डाउनलोड करें",
    iso_download_desc: "अनुशंसित। हम आधिकारिक सर्वर से प्रिज्म लिनक्स के नवीनतम संस्करण को स्वचालित रूप से डाउनलोड करेंगे।",
    iso_local: "स्थानीय फ़ाइल (.iso) का उपयोग करें",
    iso_local_desc: "अपने कंप्यूटर से पहले से डाउनलोड की गई आईएसओ फ़ाइल का चयन करें।",
    iso_select_file: "आईएसओ फ़ाइल चुनें",
    iso_selected_path: "चयनित आईएसओ: ",
    btn_start_download: "लिखना शुरू करें",
    downloading_title: "डिस्क छवि डाउनलोड हो रही है",
    speed: "गति: ",
    eta: "शेष समय: ",
    bytes_downloaded: "डाउनलोड किया गया: ",
    flasher_title: "यूएसबी फ्लैश सेटअप",
    flasher_subtitle: "प्रिज्म लिनक्स यूएसबी ड्राइव निर्माण सेटअप",
    select_drive: "यूएसबी ड्राइव का चयन करें:",
    no_drives_found: "कोई यूएसबी ड्राइव नहीं मिली। कृपया ड्राइव डालें और सूची को रीफ्रेश करें।",
    btn_refresh: "सूची रीफ्रेश करें",
    safety_title: "महत्वपूर्ण सुरक्षा चेतावनी",
    safety_desc: "चेतावनी: लिखने की प्रक्रिया से चयनित डिवाइस पर मौजूद सभी डेटा स्थायी रूप से नष्ट हो जाएगा!",
    confirm_safety_1: "मैं समझता हूं कि चयनित ड्राइव पर सभी फाइलें स्थायी रूप से हटा दी जाएंगी।",
    confirm_safety_2: "मैं पुष्टि करता हूं कि मैंने यूएसबी फ्लैश ड्राइव का चयन किया है, न कि सिस्टम हार्ड डिस्क का।",
    btn_flash: "यूएसबी पर लिखना शुरू करें",
    flashing_title: "यूएसबी पर लिखा जा रहा है...",
    flashing_desc: "कृपया कंप्यूटर बंद न करें या ड्राइव को बाहर न निकालें। इस प्रक्रिया में कुछ मिनट लग सकते हैं।",
    bytes_written: "लिखा गया: ",
    success_flash_title: "यूएसबी पर लेखन सफल रहा!",
    success_flash_desc: "प्रिज्म लिनक्स बूट करने योग्य ड्राइव तैयार है। कंप्यूटर पुनरारंभ करें और बूट मेनू में यूएसबी ड्राइव का चयन करें।",
    btn_done: "पूर्ण",
    hdd_title: "हार्ड डिस्क से सीधा बूट",
    hdd_subtitle: "अपने कंप्यूटर के बूट मेनू में प्रिज्म लिनक्स को बूट विकल्प के रूप में जोड़ें",
    hdd_warning_title: "चेतावनी: बिट लॉकर ड्राइव एन्क्रिप्शन",
    hdd_warning_desc: "यदि आपका C: ड्राइव बिट लॉकर से एन्क्रिप्टेड है, तो लिनक्स इंस्टॉलर बूट के दौरान फाइलों को नहीं पढ़ पाएगा। रीबूट से पहले विंडोज़ में बिट लॉकर को निलंबित करें!",
    hdd_step_1_title: "स्थापना फाइलें कॉपी करना",
    hdd_step_1_desc: "हम C:\\PrismLinux फ़ोल्डर बनाएंगे और उसमें कर्नेल और आईएसओ फाइलें कॉपी करेंगे।",
    hdd_step_2_title: "बूटलोडर स्थापित करना",
    hdd_step_2_desc: "हम आपके EFI सिस्टम विभाजन में आधिकारिक और हस्ताक्षरित GRUB बूटलोडर स्थापित करेंगे।",
    hdd_step_3_title: "विंडोज़ बूट मेनू में विकल्प जोड़ना",
    hdd_step_3_desc: "कंप्यूटर बूट होने पर आपको विंडोज़ या प्रिज्म लिनक्स शुरू करने का विकल्प मेनू दिखाई देगा।",
    flash_step_1_title: "डिस्क छवि तैयार करना",
    flash_step_1_desc: "हम नवीनतम संस्करण डाउनलोड करेंगे या चयनित स्थानीय फ़ाइल तैयार करेंगे।",
    flash_step_2_title: "यूएसबी ड्राइव लॉक करना",
    flash_step_2_desc: "हम सुरक्षित और विशेष लेखन के लिए बाहरी यूएसबी ड्राइव को ब्लॉक मोड में लॉक कर देंगे।",
    flash_step_3_title: "ब्लॉक दर ब्लॉक लेखन",
    flash_step_3_desc: "प्रक्रिया के तहत डेटा को ब्लॉक दर ब्लॉक चयनित ड्राइव में कॉपी किया जा रहा है।",
    confirm_hdd_1: "मैं पुष्टि करता हूं कि मेरा C: ड्राइव बिट लॉकर से एन्क्रिप्टेड नहीं है।",
    confirm_hdd_2: "मैं कंप्यूटर के बूट मेनू में स्थापना विकल्प जोड़ने की अनुमति देता हूं।",
    btn_install_hdd: "सीधे बूट सेटअप करें",
    hdd_status_working: "बूटलोडर सेटअप हो रहा है... कृपया प्रतीक्षा करें।",
    success_hdd_title: "सिस्टम सेटअप सफल!",
    success_hdd_desc: "सेटअप सफल। रीबूट करें और बूट मेनू में 'Prism Linux Live Installer' चुनें।",
    btn_reboot_now: "कंप्यूटर पुनरारंभ करें",
    error_title: "त्रुटि उत्पन्न हुई",
    btn_ok: "समझ गया",
    exit_title: "इंस्टॉलर से बाहर निकलें",
    exit_confirm_msg: "क्या आप वाकई प्रक्रिया को रोकना और इंस्टॉलर से बाहर निकलना चाहते हैं?",
    btn_yes: "हाँ, बाहर निकलें",
    btn_no: "नहीं, रुकें",
  },
};

const t = computed(() => translations[lang.value]);

const dashboardSubtitle = computed(() => {
  if (isLinux.value) {
    if (lang.value === "ua") {
      return "Ви запустили інсталятор на ОС Linux. Запишіть образ на флешку для встановлення.";
    } else if (lang.value === "de") {
      return "Sie haben den Installer unter Linux gestartet. Schreiben Sie das Image auf einen USB-Stick.";
    } else if (lang.value === "es") {
      return "Ha iniciado el instalador en Linux. Grabe la imagen en una unidad USB para la instalación.";
    } else if (lang.value === "ar") {
      return "لقد قمت بتشغيل برنامج التثبيت على نظام لينكس. يرجى كتابة الصورة على فلاشة USB للتثبيت.";
    } else if (lang.value === "zh") {
      return "您已在 Linux 系统中启动安装程序。请将镜像写入 U 盘以进行安装。";
    } else if (lang.value === "ja") {
      return "Linuxでインストーラーを起動しました。インストールするにはイメージをUSBに書き込んでください。";
    } else if (lang.value === "hi") {
      return "आपने लिनक्स पर इंस्टॉलर शुरू किया है। कृपया स्थापना के लिए छवि को यूएसबी पर लिखें।";
    } else {
      return "You are running the installer on Linux. Write the image to a USB flash drive to install.";
    }
  }
  return t.value.dashboard_subtitle;
});

function formatBytes(bytes: number): string {
  if (bytes === 0) return "0 Bytes";
  const k = 1024;
  const sizes = ["Bytes", "KB", "MB", "GB"];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + " " + sizes[i];
}

function formatSpeed(bytesPerSec: number): string {
  return formatBytes(bytesPerSec) + "/s";
}

function calculateEta(current: number, total: number, speed: number): string {
  if (speed <= 0 || total <= 0) return "Calculating...";
  const remainingBytes = total - current;
  const seconds = Math.ceil(remainingBytes / speed);
  if (seconds < 60) {
    return `${seconds}s`;
  }
  const minutes = Math.floor(seconds / 60);
  const remSec = seconds % 60;
  return `${minutes}m ${remSec}s`;
}

async function selectDashboardOption(option: "flash" | "hdd") {
  selectedMode.value = option;
  if (option === "flash") {
    refreshDrivesList();
    currentScreen.value = "flasher";
  } else {
    currentScreen.value = "hdd-installer";
  }
}

async function triggerFileSelect() {
  try {
    const path = await invoke<string>("select_local_iso");
    localIsoPath.value = path;
  } catch (err: any) {
    if (err !== "No file selected." && err !== "File selection cancelled or failed.") {
      showError(err);
    }
  }
}

function showError(msg: string) {
  errorMessage.value = msg;
}

function clearError() {
  errorMessage.value = "";
}

async function handleFlasherConfigNext() {
  if (selectedDriveNum.value === null) return;
  currentScreen.value = "iso-selector";
}

async function startFlasherExecutionChain() {
  const targetIsoPath =
    isoSource.value === "download" ? "C:\\PrismLinux\\temp_prism.iso" : localIsoPath.value;

  if (isoSource.value === "local" && !localIsoPath.value) {
    showError(lang.value === "ua" ? "Будь ласка, оберіть файл образу диска!" : "Please select an image file first!");
    return;
  }

  if (isoSource.value === "download") {
    currentScreen.value = "progress-download";
    progressBytes.value = 0;
    progressTotal.value = 0;
    progressSpeed.value = 0;

    try {
      await invoke("download_iso", {
        url: "https://prismlinux.org/download/proxy?type=primary",
        savePath: "C:\\PrismLinux\\temp_prism.iso",
      });
      runUsbFlashingDirect("C:\\PrismLinux\\temp_prism.iso");
    } catch (err: any) {
      currentScreen.value = "iso-selector";
      showError(err);
    }
  } else {
    runUsbFlashingDirect(targetIsoPath);
  }
}

async function runUsbFlashingDirect(isoPath: string) {
  if (selectedDriveNum.value === null) return;
  currentScreen.value = "progress-flash";
  progressBytes.value = 0;
  progressTotal.value = 0;
  progressSpeed.value = 0;

  try {
    await invoke("flash_usb_drive", {
      driveNum: selectedDriveNum.value,
      isoPath: isoPath,
    });
    currentScreen.value = "success-flash";
  } catch (err: any) {
    currentScreen.value = "iso-selector";
    showError(err);
  }
}

async function refreshDrivesList() {
  isRefreshingDrives.value = true;
  selectedDriveNum.value = null;
  confirmSafety1.value = false;
  confirmSafety2.value = false;
  try {
    const list = await invoke<UsbDrive[]>("get_usb_drives");
    usbDrives.value = list;
  } catch (err: any) {
    showError(err);
  } finally {
    isRefreshingDrives.value = false;
  }
}

async function startHddConfiguration() {
  const isoPath =
    isoSource.value === "download" ? "C:\\PrismLinux\\temp_prism.iso" : localIsoPath.value;

  if (isoSource.value === "download") {
    currentScreen.value = "progress-download";
    progressBytes.value = 0;
    progressTotal.value = 0;
    progressSpeed.value = 0;

    try {
      await invoke("download_iso", {
        url: "https://prismlinux.org/download/proxy?type=primary",
        savePath: "C:\\PrismLinux\\temp_prism.iso",
      });
      runHddConfigurationDirect("C:\\PrismLinux\\temp_prism.iso");
    } catch (err: any) {
      currentScreen.value = "hdd-installer";
      showError(err);
    }
  } else {
    runHddConfigurationDirect(isoPath);
  }
}

async function runHddConfigurationDirect(isoPath: string) {
  currentScreen.value = "progress-hdd";
  hddStatusText.value = t.value.hdd_status_working;

  try {
    await invoke("setup_direct_boot", { isoPath: isoPath });
    currentScreen.value = "success-hdd";
  } catch (err: any) {
    currentScreen.value = "hdd-installer";
    showError(err);
  }
}

async function rebootComputer() {
  try {
    await invoke("reboot_system");
  } catch (err: any) {
    showError(err);
  }
}

async function confirmExit() {
  try {
    await invoke("exit_app");
  } catch {
    showExitModal.value = false;
  }
}

function selectLanguage(code: "ua" | "en" | "de" | "es" | "ar" | "zh" | "ja" | "hi") {
  lang.value = code;
  isLangDropdownOpen.value = false;
}

function closeLangDropdown() {
  isLangDropdownOpen.value = false;
}

onMounted(async () => {
  isLinux.value = navigator.userAgent.toLowerCase().includes("linux");
  const systemLang = (navigator.language || "en").toLowerCase();
  if (systemLang.startsWith("uk")) {
    lang.value = "ua";
  } else if (systemLang.startsWith("de")) {
    lang.value = "de";
  } else if (systemLang.startsWith("es")) {
    lang.value = "es";
  } else if (systemLang.startsWith("ar")) {
    lang.value = "ar";
  } else if (systemLang.startsWith("zh")) {
    lang.value = "zh";
  } else if (systemLang.startsWith("ja")) {
    lang.value = "ja";
  } else if (systemLang.startsWith("hi")) {
    lang.value = "hi";
  } else {
    lang.value = "en";
  }

  unlistenDownload = await listen<ProgressEvent>("download-progress", (event) => {
    const payload = event.payload;
    if (payload.downloaded !== undefined) {
      progressBytes.value = payload.downloaded;
    }
    progressTotal.value = payload.total;
    progressSpeed.value = payload.speed;
  });

  unlistenFlash = await listen<ProgressEvent>("flash-progress", (event) => {
    const payload = event.payload;
    if (payload.written !== undefined) {
      progressBytes.value = payload.written;
    }
    progressTotal.value = payload.total;
    progressSpeed.value = payload.speed;
  });

  unlistenClose = await listen("close-requested", () => {
    showExitModal.value = true;
  });

  window.addEventListener("click", closeLangDropdown);
});

onUnmounted(() => {
  if (unlistenDownload) unlistenDownload();
  if (unlistenFlash) unlistenFlash();
  if (unlistenClose) unlistenClose();
  window.removeEventListener("click", closeLangDropdown);
});

const progressPercent = computed(() => {
  if (progressTotal.value <= 0) return 0;
  return Math.min(100, Math.round((progressBytes.value / progressTotal.value) * 100));
});

const downloadEta = computed(() => {
  return calculateEta(progressBytes.value, progressTotal.value, progressSpeed.value);
});
</script>

<template>
  <div class="app-container" :dir="lang === 'ar' ? 'rtl' : 'ltr'">
    <header class="app-header">
      <div class="logo-section">
        <div class="logo-icon"></div>
        <span class="logo-text">{{ t.title }}</span>
      </div>
      <div class="header-controls">
        <div class="custom-dropdown-container" @click.stop>
          <button class="btn-lang" :class="{ open: isLangDropdownOpen }" @click="isLangDropdownOpen = !isLangDropdownOpen">
            <svg class="globe-icon" xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="10"/><path d="M12 2a14.5 14.5 0 0 0 0 20 14.5 14.5 0 0 0 0-20"/><path d="M2 12h20"/></svg>
            <span class="lang-code-selected">{{ lang.toUpperCase() }}</span>
            <svg class="chevron-icon" xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"><path d="m6 9 6 6 6-6"/></svg>
          </button>
          <div v-if="isLangDropdownOpen" class="lang-dropdown-list">
            <div
              v-for="l in languagesList"
              :key="l.code"
              class="lang-dropdown-item"
              :class="{ active: lang === l.code }"
              @click="selectLanguage(l.code as any)"
            >
              <span class="lang-badge">{{ l.code.toUpperCase() }}</span>
              <span class="lang-name">{{ l.name }}</span>
            </div>
          </div>
        </div>
      </div>
    </header>

    <main class="main-content">
      <div v-if="currentScreen === 'dashboard'" style="text-align: center; margin-bottom: 32px;">
        <p style="font-size: 15px; color: var(--text-secondary); max-width: 600px; margin: 0 auto; line-height: 1.6;">
          {{ dashboardSubtitle }}
        </p>
      </div>

      <div v-if="currentScreen === 'dashboard'" class="cards-grid">
        <div class="action-card" @click="selectDashboardOption('flash')">
          <div class="card-icon-container">
            <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="7" y="2" width="10" height="20" rx="2" ry="2"/><path d="M12 6v4"/><path d="M12 14v4"/><path d="M9 10h6"/></svg>
          </div>
          <h2 class="card-title">{{ t.card_flash_title }}</h2>
          <p class="card-description">{{ t.card_flash_desc }}</p>
        </div>

        <div v-if="!isLinux" class="action-card secondary" @click="selectDashboardOption('hdd')">
          <div class="card-icon-container">
            <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="2" y="3" width="20" height="18" rx="2" ry="2"/><path d="M12 12h.01"/><path d="M12 16h.01"/><path d="M16 12h.01"/><path d="M16 16h.01"/><path d="M6 8h12"/></svg>
          </div>
          <h2 class="card-title">{{ t.card_hdd_title }}</h2>
          <p class="card-description">{{ t.card_hdd_desc }}</p>
        </div>
      </div>

      <div v-else-if="currentScreen === 'flasher'" class="view-panel">
        <div class="view-header">
          <h2 class="view-title">
            <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z"/></svg>
            {{ t.flasher_title }}
          </h2>
          <button class="btn-back" @click="currentScreen = 'dashboard'">
            {{ t.btn_back }}
          </button>
        </div>

        <p style="font-size: 14.5px; color: var(--text-secondary); margin-bottom: 20px;">{{ t.flasher_subtitle }}</p>

        <div class="step-list">
          <div class="step-item">
            <div class="step-number">1</div>
            <div class="step-content">
              <h4 class="step-title">{{ t.flash_step_1_title }}</h4>
              <p>{{ t.flash_step_1_desc }}</p>
            </div>
          </div>
          <div class="step-item">
            <div class="step-number">2</div>
            <div class="step-content">
              <h4 class="step-title">{{ t.flash_step_2_title }}</h4>
              <p>{{ t.flash_step_2_desc }}</p>
            </div>
          </div>
          <div class="step-item">
            <div class="step-number">3</div>
            <div class="step-content">
              <h4 class="step-title">{{ t.flash_step_3_title }}</h4>
              <p>{{ t.flash_step_3_desc }}</p>
            </div>
          </div>
        </div>

        <div class="control-group">
          <label class="control-label">{{ t.select_drive }}</label>
          <div v-if="isRefreshingDrives" style="display: flex; align-items: center; gap: 12px; padding: 12px; color: var(--text-secondary);">
            <div class="loader-spinner"></div>
            <span>{{ t.hdd_status_working }}</span>
          </div>
          <div v-else-if="usbDrives.length === 0" style="padding: 16px; border: 1px solid var(--border-color); border-radius: 6px; background: rgba(0,0,0,0.2);">
            <p style="font-size: 13.5px; color: var(--text-secondary); margin-bottom: 12px;">{{ t.no_drives_found }}</p>
            <button class="btn-secondary" style="padding: 10px 16px; font-size: 14px;" @click="refreshDrivesList">{{ t.btn_refresh }}</button>
          </div>
          <div v-else style="display: flex; gap: 12px;">
            <select class="select-custom" v-model="selectedDriveNum" style="flex: 1;">
              <option :value="null" disabled>{{ lang === 'ua' ? 'Оберіть диск...' : 'Select a drive...' }}</option>
              <option v-for="drive in usbDrives" :key="drive.Number" :value="drive.Number">
                PhysicalDrive{{ drive.Number }} - {{ drive.FriendlyName }} ({{ (drive.Size / 1024 / 1024 / 1024).toFixed(1) }} GB)
              </option>
            </select>
            <button class="btn-secondary" style="width: auto; padding: 12px 18px;" @click="refreshDrivesList">R</button>
          </div>
        </div>

        <div class="warning-panel danger" v-if="selectedDriveNum !== null">
          <div class="warning-text">
            <h4 class="warning-title">{{ t.safety_title }}</h4>
            <p>{{ t.safety_desc }}</p>
          </div>
        </div>

        <div v-if="selectedDriveNum !== null" style="margin-top: 16px; margin-bottom: 24px;">
          <div class="checkbox-container" @click="confirmSafety1 = !confirmSafety1">
            <input type="checkbox" class="checkbox-custom" :checked="confirmSafety1" @click.prevent />
            <label class="checkbox-label">{{ t.confirm_safety_1 }}</label>
          </div>
          <div class="checkbox-container" style="margin-top: 12px;" @click="confirmSafety2 = !confirmSafety2">
            <input type="checkbox" class="checkbox-custom" :checked="confirmSafety2" @click.prevent />
            <label class="checkbox-label">{{ t.confirm_safety_2 }}</label>
          </div>
        </div>

        <button class="btn-primary" :disabled="selectedDriveNum === null || !confirmSafety1 || !confirmSafety2" @click="handleFlasherConfigNext">
          {{ t.btn_next }}
        </button>
      </div>

      <div v-else-if="currentScreen === 'iso-selector'" class="view-panel">
        <div class="view-header">
          <h2 class="view-title">
            <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="10"/><path d="M12 2v20"/><path d="M17 5H9.5a3.5 3.5 0 0 0 0 7h5a3.5 3.5 0 0 1 0 7H6"/></svg>
            {{ t.iso_title }}
          </h2>
          <button class="btn-back" @click="currentScreen = selectedMode === 'flash' ? 'flasher' : 'hdd-installer'">
            {{ t.btn_back }}
          </button>
        </div>

        <div class="control-group">
          <div class="checkbox-container" @click="isoSource = 'download'">
            <input type="radio" class="checkbox-custom" name="isoSource" :checked="isoSource === 'download'" />
            <div class="checkbox-label" style="font-size: 15px; font-weight: 600; color: #fff;">
              {{ t.iso_download }}
              <div style="font-size: 13px; font-weight: normal; color: var(--text-secondary); margin-top: 4px;">
                {{ t.iso_download_desc }}
              </div>
            </div>
          </div>

          <div class="checkbox-container" style="margin-top: 24px;" @click="isoSource = 'local'">
            <input type="radio" class="checkbox-custom" name="isoSource" :checked="isoSource === 'local'" />
            <div class="checkbox-label" style="font-size: 15px; font-weight: 600; color: #fff;">
              {{ t.iso_local }}
              <div style="font-size: 13px; font-weight: normal; color: var(--text-secondary); margin-top: 4px; margin-bottom: 12px;">
                {{ t.iso_local_desc }}
              </div>

              <div v-if="isoSource === 'local'" class="file-dropzone" @click.stop="triggerFileSelect">
                <div class="file-dropzone-text">{{ t.iso_select_file }}</div>
                <div v-if="localIsoPath" class="file-dropzone-sub" style="color: var(--secondary); font-weight: bold; overflow-wrap: anywhere;">
                  {{ t.iso_selected_path }}{{ localIsoPath }}
                </div>
              </div>
            </div>
          </div>
        </div>

        <button class="btn-primary" style="margin-top: 24px;" @click="selectedMode === 'flash' ? startFlasherExecutionChain() : startHddConfiguration()">
          {{ t.btn_start_download }}
        </button>
      </div>

      <div v-else-if="currentScreen === 'progress-download'" class="view-panel">
        <div class="view-header">
          <h2 class="view-title">{{ t.downloading_title }}</h2>
        </div>

        <div class="progress-section">
          <div class="progress-header">
            <span>{{ progressPercent }}%</span>
            <span>{{ downloadEta }}</span>
          </div>

          <div class="progress-track">
            <div class="progress-fill" :style="{ width: progressPercent + '%' }"></div>
          </div>

          <div class="progress-meta">
            <span>{{ t.bytes_downloaded }}{{ formatBytes(progressBytes) }} / {{ formatBytes(progressTotal) }}</span>
            <span>{{ t.speed }}{{ formatSpeed(progressSpeed) }}</span>
          </div>
        </div>
      </div>

      <div v-else-if="currentScreen === 'progress-flash'" class="view-panel">
        <div class="view-header">
          <h2 class="view-title">{{ t.flashing_title }}</h2>
        </div>
        <p style="font-size: 14px; color: var(--text-secondary); margin-bottom: 20px;">{{ t.flashing_desc }}</p>

        <div class="progress-section">
          <div class="progress-header">
            <span>{{ progressPercent }}%</span>
            <span>{{ downloadEta }}</span>
          </div>

          <div class="progress-track">
            <div class="progress-fill" :style="{ width: progressPercent + '%' }"></div>
          </div>

          <div class="progress-meta">
            <span>{{ t.bytes_written }}{{ formatBytes(progressBytes) }} / {{ formatBytes(progressTotal) }}</span>
            <span>{{ t.speed }}{{ formatSpeed(progressSpeed) }}</span>
          </div>
        </div>
      </div>

      <div v-else-if="currentScreen === 'success-flash'" class="view-panel success-screen">
        <div class="success-icon">V</div>
        <h2 class="success-title">{{ t.success_flash_title }}</h2>
        <p class="success-description">{{ t.success_flash_desc }}</p>
        <button class="btn-primary" style="max-width: 200px;" @click="currentScreen = 'dashboard'">
          {{ t.btn_done }}
        </button>
      </div>

      <div v-else-if="currentScreen === 'hdd-installer'" class="view-panel">
        <div class="view-header">
          <h2 class="view-title">
            <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z"/></svg>
            {{ t.hdd_title }}
          </h2>
          <button class="btn-back" @click="currentScreen = 'dashboard'">
            {{ t.btn_back }}
          </button>
        </div>

        <p style="font-size: 14.5px; color: var(--text-secondary); margin-bottom: 20px;">{{ t.hdd_subtitle }}</p>

        <div class="warning-panel">
          <div class="warning-text">
            <h4 class="warning-title">{{ t.hdd_warning_title }}</h4>
            <p>{{ t.hdd_warning_desc }}</p>
          </div>
        </div>

        <div class="step-list">
          <div class="step-item">
            <div class="step-number">1</div>
            <div class="step-content">
              <h4 class="step-title">{{ t.hdd_step_1_title }}</h4>
              <p>{{ t.hdd_step_1_desc }}</p>
            </div>
          </div>
          <div class="step-item">
            <div class="step-number">2</div>
            <div class="step-content">
              <h4 class="step-title">{{ t.hdd_step_2_title }}</h4>
              <p>{{ t.hdd_step_2_desc }}</p>
            </div>
          </div>
          <div class="step-item">
            <div class="step-number">3</div>
            <div class="step-content">
              <h4 class="step-title">{{ t.hdd_step_3_title }}</h4>
              <p>{{ t.hdd_step_3_desc }}</p>
            </div>
          </div>
        </div>

        <div style="margin-top: 16px; margin-bottom: 24px;">
          <div class="checkbox-container" @click="confirmHdd1 = !confirmHdd1">
            <input type="checkbox" class="checkbox-custom" :checked="confirmHdd1" @click.prevent />
            <label class="checkbox-label">{{ t.confirm_hdd_1 }}</label>
          </div>
          <div class="checkbox-container" style="margin-top: 12px;" @click="confirmHdd2 = !confirmHdd2">
            <input type="checkbox" class="checkbox-custom" :checked="confirmHdd2" @click.prevent />
            <label class="checkbox-label">{{ t.confirm_hdd_2 }}</label>
          </div>
        </div>

        <button class="btn-primary" :disabled="!confirmHdd1 || !confirmHdd2" @click="startHddConfiguration">
          {{ t.btn_install_hdd }}
        </button>
      </div>

      <div v-else-if="currentScreen === 'progress-hdd'" class="view-panel" style="text-align: center; padding: 48px 32px;">
        <div class="loader-spinner" style="width: 48px; height: 48px; border-width: 4px; margin: 0 auto 24px auto;"></div>
        <h3 style="font-family: var(--font-sans); font-weight: 700; font-size: 20px; color: #fff; margin-bottom: 8px;">{{ t.hdd_status_working }}</h3>
        <p style="font-size: 14px; color: var(--text-secondary);">{{ hddStatusText }}</p>
      </div>

      <div v-else-if="currentScreen === 'success-hdd'" class="view-panel success-screen">
        <div class="success-icon">V</div>
        <h2 class="success-title">{{ t.success_hdd_title }}</h2>
        <p class="success-description">{{ t.success_hdd_desc }}</p>
        <button class="btn-primary" style="max-width: 250px;" @click="rebootComputer">
          {{ t.btn_reboot_now }}
        </button>
      </div>
    </main>

    <div v-if="errorMessage" style="position: fixed; inset: 0; background: rgba(6,4,10,0.85); display: flex; align-items: center; justify-content: center; z-index: 999; padding: 24px; backdrop-filter: blur(8px);">
      <div class="view-panel" style="max-width: 500px; border-color: var(--error);">
        <div style="display: flex; gap: 16px; align-items: flex-start; margin-bottom: 24px;">
          <div style="width: 40px; height: 40px; border-radius: 50%; border: 2px solid var(--error); color: var(--error); display: flex; align-items: center; justify-content: center; font-size: 20px; font-weight: bold; flex-shrink: 0;">!</div>
          <div>
            <h3 style="font-family: var(--font-sans); font-weight: 700; font-size: 18px; color: #fff; margin-bottom: 8px;">{{ t.error_title }}</h3>
            <p style="font-size: 13.5px; line-height: 1.6; color: var(--text-secondary); word-break: break-word;">{{ errorMessage }}</p>
          </div>
        </div>
        <button class="btn-primary" style="background: var(--error);" @click="clearError">
          {{ t.btn_ok }}
        </button>
      </div>
    </div>

    <div v-if="showExitModal" class="exit-overlay" style="position: fixed; inset: 0; background: rgba(6, 4, 10, 0.95); display: flex; align-items: center; justify-content: center; z-index: 9999; padding: 24px; backdrop-filter: blur(8px);">
      <div class="view-panel" style="max-width: 500px; border-color: var(--warning); text-align: center; display: flex; flex-direction: column; align-items: center; gap: 20px;">
        <svg width="120" height="120" viewBox="0 0 120 120" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" style="color: var(--primary);">
          <path d="M20 50 L35 20 L50 45" />
          <path d="M100 50 L85 20 L70 45" />
          <path d="M20 50 C20 90, 100 90, 100 50 C100 45, 90 40, 60 40 C30 40, 20 45, 20 50 Z" />
          <path d="M35 60 C40 65, 45 65, 50 60" />
          <path d="M70 60 C75 65, 80 65, 85 60" />
          <path d="M42 66 L42 85" stroke="#1bd3f2" stroke-width="3" />
          <circle cx="42" cy="88" r="2" fill="#1bd3f2" stroke="none" />
          <path d="M78 66 L78 85" stroke="#1bd3f2" stroke-width="3" />
          <circle cx="78" cy="88" r="2" fill="#1bd3f2" stroke="none" />
          <polygon points="57,68 63,68 60,71" fill="currentColor" />
          <path d="M54 77 C57 80, 60 80, 60 77 C60 80, 63 80, 66 77" />
          <path d="M15 70 L5 68" />
          <path d="M15 75 L3 76" />
          <path d="M105 70 L115 68" />
          <path d="M105 75 L117 76" />
        </svg>
        <h3 style="font-family: var(--font-sans); font-weight: 700; font-size: 20px; color: #fff;">{{ t.exit_title }}</h3>
        <p style="font-size: 14px; line-height: 1.6; color: var(--text-secondary);">{{ t.exit_confirm_msg }}</p>
        <div style="display: flex; gap: 12px; width: 100%; margin-top: 12px;">
          <button class="btn-primary" style="background: var(--error); border-color: var(--error); color: #fff;" @click="confirmExit">
            {{ t.btn_yes }}
          </button>
          <button class="btn-secondary" @click="showExitModal = false">
            {{ t.btn_no }}
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<style>
input[type="radio"].checkbox-custom {
  border-radius: 50%;
}

input[type="radio"].checkbox-custom:checked::after {
  content: '';
  width: 10px;
  height: 10px;
  background: #121212;
  border-radius: 50%;
  position: absolute;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
}
</style>