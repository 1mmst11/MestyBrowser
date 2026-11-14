(function () {
  "use strict";

  const LANGUAGES = [
    { code: "en", name: "English" },
    { code: "ru", name: "Русский" },
    { code: "uk", name: "Українська" },
    { code: "de", name: "Deutsch" },
    { code: "fr", name: "Français" },
    { code: "es", name: "Español" },
    { code: "it", name: "Italiano" },
    { code: "pt", name: "Português" },
    { code: "pl", name: "Polski" },
    { code: "tr", name: "Türkçe" },
    { code: "cs", name: "Čeština" },
    { code: "nl", name: "Nederlands" },
    { code: "zh", name: "中文" },
    { code: "ja", name: "日本語" }
  ];

  const STRINGS = {
    en: {
      newTab: "New tab", find: "Find in page", zoom: "Zoom", reset: "Reset",
      reopen: "Reopen closed tab", fullscreen: "Fullscreen", settings: "Settings",
      search: "Search {engine} or enter address", frequent: "Frequently visited",
      tagline: "A quiet place to browse.", done: "Done", theme: "Theme",
      accent: "Accent color", background: "New tab background", language: "Language",
      interface: "Interface", about: "About", autoSave: "Changes save automatically.",
      resetDefaults: "Reset to defaults", chooseImage: "Choose image…", removeImage: "Remove",
      compact: "Compact tabs", rounded: "Rounded corners", animations: "Animations",
      searchEngine: "Search engine", menu: "Menu", back: "Back", forward: "Forward",
      reload: "Reload",
      customizeHome: "Customize home", editHint: "Drag anything, rotate with the round handle, then save.",
      addSite: "Add site", siteAddress: "Site address", add: "Add", save: "Save",
      cancel: "Cancel", remove: "Remove",
      startEditing: "Start editing", backgroundMusic: "Background music",
      chooseAudio: "Choose MP3…", audioHint: "Plays on the new tab page. MP3, OGG or WAV.",
      chooseVideo: "Choose video…", backgroundHint: "Use a wallpaper, your own image, or a looping video.",
      resetLayout: "Reset layout", rotate: "Rotate"
    },
    ru: {
      newTab: "Новая вкладка", find: "Найти на странице", zoom: "Масштаб", reset: "Сбросить",
      reopen: "Открыть закрытую вкладку", fullscreen: "Полный экран", settings: "Настройки",
      search: "Поиск в {engine} или введите адрес", frequent: "Часто посещаемые",
      tagline: "Тихое место для интернета.", done: "Готово", theme: "Тема",
      accent: "Акцентный цвет", background: "Фон новой вкладки", language: "Язык",
      interface: "Интерфейс", about: "О браузере", autoSave: "Изменения сохраняются автоматически.",
      resetDefaults: "Сбросить настройки", chooseImage: "Выбрать изображение…", removeImage: "Удалить",
      compact: "Компактные вкладки", rounded: "Скруглённые углы", animations: "Анимации",
      searchEngine: "Поисковая система", menu: "Меню", back: "Назад", forward: "Вперёд",
      reload: "Обновить",
      customizeHome: "Настроить главную", editHint: "Перетаскивайте что угодно, поворачивайте круглым маркером, затем сохраните.",
      addSite: "Добавить сайт", siteAddress: "Адрес сайта", add: "Добавить", save: "Сохранить",
      cancel: "Отмена", remove: "Удалить",
      startEditing: "Начать настройку", backgroundMusic: "Фоновая музыка",
      chooseAudio: "Выбрать MP3…", audioHint: "Играет на странице новой вкладки. MP3, OGG или WAV.",
      chooseVideo: "Выбрать видео…", backgroundHint: "Обои, своё изображение или зацикленное видео.",
      resetLayout: "Сбросить макет", rotate: "Повернуть"
    },
    uk: {
      newTab: "Нова вкладка", find: "Знайти на сторінці", zoom: "Масштаб", reset: "Скинути",
      reopen: "Відкрити закриту вкладку", fullscreen: "Повний екран", settings: "Налаштування",
      search: "Пошук у {engine} або введіть адресу", frequent: "Часто відвідувані",
      tagline: "Затишне місце для інтернету.", done: "Готово", theme: "Тема",
      accent: "Акцентний колір", background: "Фон нової вкладки", language: "Мова",
      interface: "Інтерфейс", about: "Про браузер", autoSave: "Зміни зберігаються автоматично.",
      resetDefaults: "Скинути налаштування", chooseImage: "Вибрати зображення…", removeImage: "Видалити",
      compact: "Компактні вкладки", rounded: "Заокруглені кути", animations: "Анімації",
      searchEngine: "Пошукова система", menu: "Меню", back: "Назад", forward: "Уперед",
      reload: "Оновити",
      customizeHome: "Налаштувати головну", editHint: "Перетягуйте будь-що, повертайте круглим маркером, потім збережіть.",
      addSite: "Додати сайт", siteAddress: "Адреса сайту", add: "Додати", save: "Зберегти",
      cancel: "Скасувати", remove: "Видалити",
      startEditing: "Почати налаштування", backgroundMusic: "Фонова музика",
      chooseAudio: "Вибрати MP3…", audioHint: "Грає на сторінці нової вкладки. MP3, OGG або WAV.",
      chooseVideo: "Вибрати відео…", backgroundHint: "Шпалери, власне зображення або відео з циклом.",
      resetLayout: "Скинути макет", rotate: "Повернути"
    },
    de: {
      newTab: "Neuer Tab", find: "Auf Seite suchen", zoom: "Zoom", reset: "Zurücksetzen",
      reopen: "Geschlossenen Tab wieder öffnen", fullscreen: "Vollbild", settings: "Einstellungen",
      search: "Mit {engine} suchen oder Adresse eingeben", frequent: "Häufig besucht",
      tagline: "Ein ruhiger Ort zum Surfen.", done: "Fertig", theme: "Design",
      accent: "Akzentfarbe", background: "Hintergrund des neuen Tabs", language: "Sprache",
      interface: "Oberfläche", about: "Über", autoSave: "Änderungen werden automatisch gespeichert.",
      resetDefaults: "Auf Standard zurücksetzen", chooseImage: "Bild auswählen…", removeImage: "Entfernen",
      compact: "Kompakte Tabs", rounded: "Abgerundete Ecken", animations: "Animationen",
      searchEngine: "Suchmaschine", menu: "Menü", back: "Zurück", forward: "Vorwärts",
      reload: "Neu laden",
      customizeHome: "Startseite anpassen", editHint: "Ziehe alles, drehe mit dem runden Griff und speichere.",
      addSite: "Seite hinzufügen", siteAddress: "Seitenadresse", add: "Hinzufügen", save: "Speichern",
      cancel: "Abbrechen", remove: "Entfernen",
      startEditing: "Bearbeitung starten", backgroundMusic: "Hintergrundmusik",
      chooseAudio: "MP3 auswählen…", audioHint: "Spielt auf der Neuer-Tab-Seite. MP3, OGG oder WAV.",
      chooseVideo: "Video auswählen…", backgroundHint: "Verwende ein Hintergrundbild, dein eigenes Bild oder ein Schleifenvideo.",
      resetLayout: "Layout zurücksetzen", rotate: "Drehen"
    },
    fr: {
      newTab: "Nouvel onglet", find: "Rechercher dans la page", zoom: "Zoom", reset: "Réinitialiser",
      reopen: "Rouvrir l’onglet fermé", fullscreen: "Plein écran", settings: "Paramètres",
      search: "Rechercher avec {engine} ou saisir une adresse", frequent: "Sites fréquents",
      tagline: "Un endroit calme pour naviguer.", done: "Terminé", theme: "Thème",
      accent: "Couleur d’accent", background: "Fond du nouvel onglet", language: "Langue",
      interface: "Interface", about: "À propos", autoSave: "Les modifications sont enregistrées automatiquement.",
      resetDefaults: "Réinitialiser", chooseImage: "Choisir une image…", removeImage: "Supprimer",
      compact: "Onglets compacts", rounded: "Coins arrondis", animations: "Animations",
      searchEngine: "Moteur de recherche", menu: "Menu", back: "Retour", forward: "Suivant",
      reload: "Actualiser",
      customizeHome: "Personnaliser l’accueil", editHint: "Déplacez tout, faites pivoter avec la poignée ronde, puis enregistrez.",
      addSite: "Ajouter un site", siteAddress: "Adresse du site", add: "Ajouter", save: "Enregistrer",
      cancel: "Annuler", remove: "Retirer",
      startEditing: "Commencer l'édition", backgroundMusic: "Musique de fond",
      chooseAudio: "Choisir un MP3…", audioHint: "Joue sur la page nouvel onglet. MP3, OGG ou WAV.",
      chooseVideo: "Choisir une vidéo…", backgroundHint: "Utilisez un fond, votre image ou une vidéo en boucle.",
      resetLayout: "Réinitialiser la disposition", rotate: "Pivoter"
    },
    es: {
      newTab: "Nueva pestaña", find: "Buscar en la página", zoom: "Zoom", reset: "Restablecer",
      reopen: "Reabrir pestaña cerrada", fullscreen: "Pantalla completa", settings: "Ajustes",
      search: "Buscar en {engine} o escribir una dirección", frequent: "Sitios frecuentes",
      tagline: "Un lugar tranquilo para navegar.", done: "Hecho", theme: "Tema",
      accent: "Color de acento", background: "Fondo de nueva pestaña", language: "Idioma",
      interface: "Interfaz", about: "Acerca de", autoSave: "Los cambios se guardan automáticamente.",
      resetDefaults: "Restablecer valores", chooseImage: "Elegir imagen…", removeImage: "Quitar",
      compact: "Pestañas compactas", rounded: "Esquinas redondeadas", animations: "Animaciones",
      searchEngine: "Motor de búsqueda", menu: "Menú", back: "Atrás", forward: "Adelante",
      reload: "Recargar",
      customizeHome: "Personalizar inicio", editHint: "Arrastra cualquier cosa, gira con el tirador redondo y guarda.",
      addSite: "Añadir sitio", siteAddress: "Dirección del sitio", add: "Añadir", save: "Guardar",
      cancel: "Cancelar", remove: "Quitar",
      startEditing: "Empezar a editar", backgroundMusic: "Música de fondo",
      chooseAudio: "Elegir MP3…", audioHint: "Suena en la página de nueva pestaña. MP3, OGG o WAV.",
      chooseVideo: "Elegir vídeo…", backgroundHint: "Usa un fondo, tu propia imagen o un vídeo en bucle.",
      resetLayout: "Restablecer diseño", rotate: "Girar"
    },
    it: {
      newTab: "Nuova scheda", find: "Trova nella pagina", zoom: "Zoom", reset: "Reimposta",
      reopen: "Riapri scheda chiusa", fullscreen: "Schermo intero", settings: "Impostazioni",
      search: "Cerca con {engine} o inserisci un indirizzo", frequent: "Siti frequenti",
      tagline: "Un posto tranquillo per navigare.", done: "Fatto", theme: "Tema",
      accent: "Colore accento", background: "Sfondo nuova scheda", language: "Lingua",
      interface: "Interfaccia", about: "Informazioni", autoSave: "Le modifiche vengono salvate automaticamente.",
      resetDefaults: "Ripristina impostazioni", chooseImage: "Scegli immagine…", removeImage: "Rimuovi",
      compact: "Schede compatte", rounded: "Angoli arrotondati", animations: "Animazioni",
      searchEngine: "Motore di ricerca", menu: "Menu", back: "Indietro", forward: "Avanti",
      reload: "Ricarica",
      customizeHome: "Personalizza home", editHint: "Trascina qualsiasi cosa, ruota con la maniglia rotonda, poi salva.",
      addSite: "Aggiungi sito", siteAddress: "Indirizzo del sito", add: "Aggiungi", save: "Salva",
      cancel: "Annulla", remove: "Rimuovi",
      startEditing: "Inizia a modificare", backgroundMusic: "Musica di sottofondo",
      chooseAudio: "Scegli MP3…", audioHint: "Riproduce nella pagina nuova scheda. MP3, OGG o WAV.",
      chooseVideo: "Scegli video…", backgroundHint: "Usa uno sfondo, un'immagine o un video in loop.",
      resetLayout: "Reimposta layout", rotate: "Ruota"
    },
    pt: {
      newTab: "Nova aba", find: "Localizar na página", zoom: "Zoom", reset: "Redefinir",
      reopen: "Reabrir aba fechada", fullscreen: "Tela cheia", settings: "Configurações",
      search: "Pesquisar no {engine} ou digitar um endereço", frequent: "Sites frequentes",
      tagline: "Um lugar tranquilo para navegar.", done: "Concluído", theme: "Tema",
      accent: "Cor de destaque", background: "Fundo da nova aba", language: "Idioma",
      interface: "Interface", about: "Sobre", autoSave: "As alterações são salvas automaticamente.",
      resetDefaults: "Restaurar padrões", chooseImage: "Escolher imagem…", removeImage: "Remover",
      compact: "Abas compactas", rounded: "Cantos arredondados", animations: "Animações",
      searchEngine: "Mecanismo de busca", menu: "Menu", back: "Voltar", forward: "Avançar",
      reload: "Recarregar",
      customizeHome: "Personalizar página inicial", editHint: "Arraste qualquer coisa, gire com a alça redonda e salve.",
      addSite: "Adicionar site", siteAddress: "Endereço do site", add: "Adicionar", save: "Salvar",
      cancel: "Cancelar", remove: "Remover",
      startEditing: "Começar a editar", backgroundMusic: "Música de fundo",
      chooseAudio: "Escolher MP3…", audioHint: "Toca na página de nova aba. MP3, OGG ou WAV.",
      chooseVideo: "Escolher vídeo…", backgroundHint: "Use um plano de fundo, sua imagem ou um vídeo em loop.",
      resetLayout: "Redefinir layout", rotate: "Girar"
    },
    pl: {
      newTab: "Nowa karta", find: "Znajdź na stronie", zoom: "Powiększenie", reset: "Resetuj",
      reopen: "Otwórz zamkniętą kartę", fullscreen: "Pełny ekran", settings: "Ustawienia",
      search: "Szukaj w {engine} lub wpisz adres", frequent: "Często odwiedzane",
      tagline: "Spokojne miejsce do przeglądania.", done: "Gotowe", theme: "Motyw",
      accent: "Kolor akcentu", background: "Tło nowej karty", language: "Język",
      interface: "Interfejs", about: "O programie", autoSave: "Zmiany zapisują się automatycznie.",
      resetDefaults: "Przywróć domyślne", chooseImage: "Wybierz obraz…", removeImage: "Usuń",
      compact: "Kompaktowe karty", rounded: "Zaokrąglone rogi", animations: "Animacje",
      searchEngine: "Wyszukiwarka", menu: "Menu", back: "Wstecz", forward: "Dalej",
      reload: "Odśwież",
      customizeHome: "Dostosuj stronę startową", editHint: "Przeciągaj dowolne elementy, obracaj okrągłym uchwytem, potem zapisz.",
      addSite: "Dodaj stronę", siteAddress: "Adres strony", add: "Dodaj", save: "Zapisz",
      cancel: "Anuluj", remove: "Usuń",
      startEditing: "Rozpocznij edycję", backgroundMusic: "Muzyka w tle",
      chooseAudio: "Wybierz MP3…", audioHint: "Gra na stronie nowej karty. MP3, OGG lub WAV.",
      chooseVideo: "Wybierz wideo…", backgroundHint: "Użyj tapety, własnego obrazu lub zapętlonego wideo.",
      resetLayout: "Resetuj układ", rotate: "Obróć"
    },
    tr: {
      newTab: "Yeni sekme", find: "Sayfada bul", zoom: "Yakınlaştırma", reset: "Sıfırla",
      reopen: "Kapatılan sekmeyi aç", fullscreen: "Tam ekran", settings: "Ayarlar",
      search: "{engine} ile ara veya adres gir", frequent: "Sık ziyaret edilenler",
      tagline: "Gezinmek için sakin bir yer.", done: "Bitti", theme: "Tema",
      accent: "Vurgu rengi", background: "Yeni sekme arka planı", language: "Dil",
      interface: "Arayüz", about: "Hakkında", autoSave: "Değişiklikler otomatik kaydedilir.",
      resetDefaults: "Varsayılanlara dön", chooseImage: "Görsel seç…", removeImage: "Kaldır",
      compact: "Kompakt sekmeler", rounded: "Yuvarlak köşeler", animations: "Animasyonlar",
      searchEngine: "Arama motoru", menu: "Menü", back: "Geri", forward: "İleri",
      reload: "Yenile",
      customizeHome: "Ana sayfayı özelleştir", editHint: "Her şeyi sürükleyin, yuvarlak tutamaçla döndürün, sonra kaydedin.",
      addSite: "Site ekle", siteAddress: "Site adresi", add: "Ekle", save: "Kaydet",
      cancel: "İptal", remove: "Kaldır",
      startEditing: "Düzenlemeye başla", backgroundMusic: "Arka plan müziği",
      chooseAudio: "MP3 seç…", audioHint: "Yeni sekme sayfasında çalar. MP3, OGG veya WAV.",
      chooseVideo: "Video seç…", backgroundHint: "Duvar kağıdı, kendi görselin veya döngüsel video kullan.",
      resetLayout: "Düzeni sıfırla", rotate: "Döndür"
    },
    cs: {
      newTab: "Nová karta", find: "Hledat na stránce", zoom: "Přiblížení", reset: "Obnovit",
      reopen: "Znovu otevřít zavřenou kartu", fullscreen: "Celá obrazovka", settings: "Nastavení",
      search: "Hledat na {engine} nebo zadat adresu", frequent: "Často navštěvované",
      tagline: "Klidné místo k prohlížení.", done: "Hotovo", theme: "Motiv",
      accent: "Barva zvýraznění", background: "Pozadí nové karty", language: "Jazyk",
      interface: "Rozhraní", about: "O aplikaci", autoSave: "Změny se ukládají automaticky.",
      resetDefaults: "Obnovit výchozí", chooseImage: "Vybrat obrázek…", removeImage: "Odebrat",
      compact: "Kompaktní karty", rounded: "Zaoblené rohy", animations: "Animace",
      searchEngine: "Vyhledávač", menu: "Menu", back: "Zpět", forward: "Vpřed",
      reload: "Obnovit",
      customizeHome: "Upravit domovskou stránku", editHint: "Přetahujte cokoli, otáčejte kulatým držadlem a poté uložte.",
      addSite: "Přidat stránku", siteAddress: "Adresa stránky", add: "Přidat", save: "Uložit",
      cancel: "Zrušit", remove: "Odebrat",
      startEditing: "Začít upravovat", backgroundMusic: "Hudba na pozadí",
      chooseAudio: "Vybrat MP3…", audioHint: "Hraje na stránce nové karty. MP3, OGG nebo WAV.",
      chooseVideo: "Vybrat video…", backgroundHint: "Použijte tapetu, vlastní obrázek nebo opakované video.",
      resetLayout: "Obnovit rozvržení", rotate: "Otočit"
    },
    nl: {
      newTab: "Nieuw tabblad", find: "Zoeken op pagina", zoom: "Zoom", reset: "Herstellen",
      reopen: "Gesloten tabblad heropenen", fullscreen: "Volledig scherm", settings: "Instellingen",
      search: "Zoeken met {engine} of adres invoeren", frequent: "Vaak bezocht",
      tagline: "Een rustige plek om te browsen.", done: "Klaar", theme: "Thema",
      accent: "Accentkleur", background: "Achtergrond nieuw tabblad", language: "Taal",
      interface: "Interface", about: "Over", autoSave: "Wijzigingen worden automatisch opgeslagen.",
      resetDefaults: "Standaardwaarden herstellen", chooseImage: "Afbeelding kiezen…", removeImage: "Verwijderen",
      compact: "Compacte tabbladen", rounded: "Afgeronde hoeken", animations: "Animaties",
      searchEngine: "Zoekmachine", menu: "Menu", back: "Terug", forward: "Vooruit",
      reload: "Vernieuwen",
      customizeHome: "Startpagina aanpassen", editHint: "Sleep alles, draai met de ronde greep en sla op.",
      addSite: "Site toevoegen", siteAddress: "Siteadres", add: "Toevoegen", save: "Opslaan",
      cancel: "Annuleren", remove: "Verwijderen",
      startEditing: "Bewerken starten", backgroundMusic: "Achtergrondmuziek",
      chooseAudio: "MP3 kiezen…", audioHint: "Speelt op de nieuwe-tabpagina. MP3, OGG of WAV.",
      chooseVideo: "Video kiezen…", backgroundHint: "Gebruik een achtergrond, je eigen afbeelding of een loopvideo.",
      resetLayout: "Indeling resetten", rotate: "Draaien"
    },
    zh: {
      newTab: "新标签页", find: "在页面中查找", zoom: "缩放", reset: "重置",
      reopen: "重新打开已关闭的标签页", fullscreen: "全屏", settings: "设置",
      search: "使用 {engine} 搜索或输入网址", frequent: "经常访问",
      tagline: "安静的浏览空间。", done: "完成", theme: "主题",
      accent: "强调色", background: "新标签页背景", language: "语言",
      interface: "界面", about: "关于", autoSave: "更改会自动保存。",
      resetDefaults: "恢复默认设置", chooseImage: "选择图片…", removeImage: "移除",
      compact: "紧凑标签页", rounded: "圆角", animations: "动画",
      searchEngine: "搜索引擎", menu: "菜单", back: "后退", forward: "前进",
      reload: "刷新",
      customizeHome: "自定义主页", editHint: "拖动任意元素，用圆形手柄旋转，然后保存。",
      addSite: "添加网站", siteAddress: "网站地址", add: "添加", save: "保存",
      cancel: "取消", remove: "移除",
      startEditing: "开始编辑", backgroundMusic: "背景音乐",
      chooseAudio: "选择 MP3…", audioHint: "在新标签页播放。支持 MP3、OGG 或 WAV。",
      chooseVideo: "选择视频…", backgroundHint: "使用壁纸、自定义图片或循环视频。",
      resetLayout: "重置布局", rotate: "旋转"
    },
    ja: {
      newTab: "新しいタブ", find: "ページ内を検索", zoom: "ズーム", reset: "リセット",
      reopen: "閉じたタブを開く", fullscreen: "全画面", settings: "設定",
      search: "{engine} で検索、またはアドレスを入力", frequent: "よくアクセスするサイト",
      tagline: "静かなブラウジングの場所。", done: "完了", theme: "テーマ",
      accent: "アクセントカラー", background: "新しいタブの背景", language: "言語",
      interface: "インターフェース", about: "について", autoSave: "変更は自動的に保存されます。",
      resetDefaults: "既定値に戻す", chooseImage: "画像を選択…", removeImage: "削除",
      compact: "コンパクトタブ", rounded: "角を丸くする", animations: "アニメーション",
      searchEngine: "検索エンジン", menu: "メニュー", back: "戻る", forward: "進む",
      reload: "再読み込み",
      customizeHome: "ホームをカスタマイズ", editHint: "何でもドラッグし、丸いハンドルで回転してから保存します。",
      addSite: "サイトを追加", siteAddress: "サイトのアドレス", add: "追加", save: "保存",
      cancel: "キャンセル", remove: "削除",
      startEditing: "編集を開始", backgroundMusic: "BGM",
      chooseAudio: "MP3 を選択…", audioHint: "新しいタブページで再生します。MP3、OGG、WAV に対応。",
      chooseVideo: "動画を選択…", backgroundHint: "壁紙、自分の画像、またはループ動画を使えます。",
      resetLayout: "レイアウトをリセット", rotate: "回転"
    }
  };

  function translate(language, key, vars) {
    const table = STRINGS[language] || STRINGS.en;
    let text = table[key] || STRINGS.en[key] || key;
    if (vars) {
      Object.keys(vars).forEach((name) => {
        text = text.split("{" + name + "}").join(vars[name]);
      });
    }
    return text;
  }

  function apply(language, root) {
    const scope = root || document;
    const lang = STRINGS[language] ? language : "en";
    if (scope.documentElement) scope.documentElement.lang = lang;
    scope.querySelectorAll("[data-i18n]").forEach((element) => {
      element.textContent = translate(lang, element.dataset.i18n);
    });
    scope.querySelectorAll("[data-i18n-title]").forEach((element) => {
      element.title = translate(lang, element.dataset.i18nTitle);
    });
    scope.querySelectorAll("[data-i18n-aria]").forEach((element) => {
      element.setAttribute("aria-label", translate(lang, element.dataset.i18nAria));
    });
    scope.querySelectorAll("[data-i18n-placeholder]").forEach((element) => {
      element.placeholder = translate(lang, element.dataset.i18nPlaceholder);
    });
  }

  window.MESTY_LANGS = LANGUAGES;
  window.mestyT = translate;
  window.mestyApplyI18n = apply;
})();
