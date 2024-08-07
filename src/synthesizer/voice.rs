#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum Voice {
    AfZaAdriNeural,
    AmEtAmehaNeural,
    ArAeFatimaNeural,
    ArBhAliNeural,
    ArDzAminaNeural,
    ArEgSalmaNeural,
    ArIqBasselNeural,
    ArJoSanaNeural,
    ArKwFahedNeural,
    ArLyImanNeural,
    ArMaJamalNeural,
    ArQaAmalNeural,
    ArSaHamedNeural,
    ArSyAmanyNeural,
    ArTnHediNeural,
    ArYeMaryamNeural,
    BgBgBorislavNeural,
    BnBdNabanitaNeural,
    BnInBashkarNeural,
    CaEsJoanaNeural,
    CsCzAntoninNeural,
    CyGbAledNeural,
    DaDkChristelNeural,
    DeAtIngridNeural,
    DeChJanNeural,
    DeDeKatjaNeural,
    ElGrAthinaNeural,
    EnAuNatashaNeural,
    EnCaClaraNeural,
    EnGbLibbyNeural,
    EnHkSamNeural,
    EnIeConnorNeural,
    EnInNeerjaNeural,
    EnKeAsiliaNeural,
    EnNgAbeoNeural,
    EnNzMitchellNeural,
    EnPhJamesNeural,
    EnSgLunaNeural,
    EnTzElimuNeural,
    #[default]
    EnUsJennyNeural,
    EnZaLeahNeural,
    EsArElenaNeural,
    EsBoMarceloNeural,
    EsClCatalinaNeural,
    EsCoGonzaloNeural,
    EsCrJuanNeural,
    EsCuBelkysNeural,
    EsDoEmilioNeural,
    EsEcAndreaNeural,
    EsEsAlvaroNeural,
    EsGqJavierNeural,
    EsGtAndresNeural,
    EsHnCarlosNeural,
    EsMxDaliaNeural,
    EsNiFedericoNeural,
    EsPaMargaritaNeural,
    EsPeAlexNeural,
    EsPrKarinaNeural,
    EsPyMarioNeural,
    EsSvLorenaNeural,
    EsUsAlonsoNeural,
    EsUyMateoNeural,
    EsVePaolaNeural,
    EtEeAnuNeural,
    FaIrDilaraNeural,
    FiFiSelmaNeural,
    FilPhAngeloNeural,
    FrBeCharlineNeural,
    FrCaSylvieNeural,
    FrChArianeNeural,
    FrFrDeniseNeural,
    GaIeColmNeural,
    GlEsRoiNeural,
    GuInDhwaniNeural,
    HeIlAvriNeural,
    HiInMadhurNeural,
    HrHrGabrijelaNeural,
    HuHuNoemiNeural,
    IdIdArdiNeural,
    IsIsGudrunNeural,
    ItItIsabellaNeural,
    JaJpNanamiNeural,
    JvIdDimasNeural,
    KkKzAigulNeural,
    KmKhPisethNeural,
    KnInGaganNeural,
    KoKrSunHiNeural,
    LoLaChanthavongNeural,
    LtLtLeonasNeural,
    LvLvEveritaNeural,
    MkMkAleksandarNeural,
    MlInMidhunNeural,
    MrInAarohiNeural,
    MsMyOsmanNeural,
    MtMtGraceNeural,
    MyMmNilarNeural,
    NbNoPernilleNeural,
    NlBeArnaudNeural,
    NlNlColetteNeural,
    PlPlAgnieszkaNeural,
    PsAfGulNawazNeural,
    PtBrFranciscaNeural,
    PtPtDuarteNeural,
    RoRoAlinaNeural,
    RuRuSvetlanaNeural,
    SiLkSameeraNeural,
    SkSkLukasNeural,
    SlSiPetraNeural,
    SoSoMuuseNeural,
    SrRsNicholasNeural,
    SuIdJajangNeural,
    SvSeSofieNeural,
    SwKeRafikiNeural,
    SwTzDaudiNeural,
    TaInPallaviNeural,
    TaLkKumarNeural,
    TaSgAnbuNeural,
    TeInMohanNeural,
    ThThPremwadeeNeural,
    TrTrAhmetNeural,
    UkUaOstapNeural,
    UrInGulNeural,
    UrPkAsadNeural,
    UzUzMadinaNeural,
    ViVnHoaiMyNeural,
    ZhCnXiaoxiaoNeural,
    ZhHkHiuMaanNeural,
    ZhTwHsiaoChenNeural,
    ZuZaThandoNeural,
    Specific(&'static str),
}

impl Voice {
    pub fn as_str(&self) -> &'static str {
        match *self {
            Voice::AfZaAdriNeural => "af-ZA-AdriNeural",
            Voice::AmEtAmehaNeural => "am-ET-AmehaNeural",
            Voice::ArAeFatimaNeural => "ar-AE-FatimaNeural",
            Voice::ArBhAliNeural => "ar-BH-AliNeural",
            Voice::ArDzAminaNeural => "ar-DZ-AminaNeural",
            Voice::ArEgSalmaNeural => "ar-EG-SalmaNeural",
            Voice::ArIqBasselNeural => "ar-IQ-BasselNeural",
            Voice::ArJoSanaNeural => "ar-JO-SanaNeural",
            Voice::ArKwFahedNeural => "ar-KW-FahedNeural",
            Voice::ArLyImanNeural => "ar-LY-ImanNeural",
            Voice::ArMaJamalNeural => "ar-MA-JamalNeural",
            Voice::ArQaAmalNeural => "ar-QA-AmalNeural",
            Voice::ArSaHamedNeural => "ar-SA-HamedNeural",
            Voice::ArSyAmanyNeural => "ar-SY-AmanyNeural",
            Voice::ArTnHediNeural => "ar-TN-HediNeural",
            Voice::ArYeMaryamNeural => "ar-YE-MaryamNeural",
            Voice::BgBgBorislavNeural => "bg-BG-BorislavNeural",
            Voice::BnBdNabanitaNeural => "bn-BD-NabanitaNeural",
            Voice::BnInBashkarNeural => "bn-IN-BashkarNeural",
            Voice::CaEsJoanaNeural => "ca-ES-JoanaNeural",
            Voice::CsCzAntoninNeural => "cs-CZ-AntoninNeural",
            Voice::CyGbAledNeural => "cy-GB-AledNeural",
            Voice::DaDkChristelNeural => "da-DK-ChristelNeural",
            Voice::DeAtIngridNeural => "de-AT-IngridNeural",
            Voice::DeChJanNeural => "de-CH-JanNeural",
            Voice::DeDeKatjaNeural => "de-DE-KatjaNeural",
            Voice::ElGrAthinaNeural => "el-GR-AthinaNeural",
            Voice::EnAuNatashaNeural => "en-AU-NatashaNeural",
            Voice::EnCaClaraNeural => "en-CA-ClaraNeural",
            Voice::EnGbLibbyNeural => "en-GB-LibbyNeural",
            Voice::EnHkSamNeural => "en-HK-SamNeural",
            Voice::EnIeConnorNeural => "en-IE-ConnorNeural",
            Voice::EnInNeerjaNeural => "en-IN-NeerjaNeural",
            Voice::EnKeAsiliaNeural => "en-KE-AsiliaNeural",
            Voice::EnNgAbeoNeural => "en-NG-AbeoNeural",
            Voice::EnNzMitchellNeural => "en-NZ-MitchellNeural",
            Voice::EnPhJamesNeural => "en-PH-JamesNeural",
            Voice::EnSgLunaNeural => "en-SG-LunaNeural",
            Voice::EnTzElimuNeural => "en-TZ-ElimuNeural",
            Voice::EnUsJennyNeural => "en-US-JennyNeural",
            Voice::EnZaLeahNeural => "en-ZA-LeahNeural",
            Voice::EsArElenaNeural => "es-AR-ElenaNeural",
            Voice::EsBoMarceloNeural => "es-BO-MarceloNeural",
            Voice::EsClCatalinaNeural => "es-CL-CatalinaNeural",
            Voice::EsCoGonzaloNeural => "es-CO-GonzaloNeural",
            Voice::EsCrJuanNeural => "es-CR-JuanNeural",
            Voice::EsCuBelkysNeural => "es-CU-BelkysNeural",
            Voice::EsDoEmilioNeural => "es-DO-EmilioNeural",
            Voice::EsEcAndreaNeural => "es-EC-AndreaNeural",
            Voice::EsEsAlvaroNeural => "es-ES-AlvaroNeural",
            Voice::EsGqJavierNeural => "es-GQ-JavierNeural",
            Voice::EsGtAndresNeural => "es-GT-AndresNeural",
            Voice::EsHnCarlosNeural => "es-HN-CarlosNeural",
            Voice::EsMxDaliaNeural => "es-MX-DaliaNeural",
            Voice::EsNiFedericoNeural => "es-NI-FedericoNeural",
            Voice::EsPaMargaritaNeural => "es-PA-MargaritaNeural",
            Voice::EsPeAlexNeural => "es-PE-AlexNeural",
            Voice::EsPrKarinaNeural => "es-PR-KarinaNeural",
            Voice::EsPyMarioNeural => "es-PY-MarioNeural",
            Voice::EsSvLorenaNeural => "es-SV-LorenaNeural",
            Voice::EsUsAlonsoNeural => "es-US-AlonsoNeural",
            Voice::EsUyMateoNeural => "es-UY-MateoNeural",
            Voice::EsVePaolaNeural => "es-VE-PaolaNeural",
            Voice::EtEeAnuNeural => "et-EE-AnuNeural",
            Voice::FaIrDilaraNeural => "fa-IR-DilaraNeural",
            Voice::FiFiSelmaNeural => "fi-FI-SelmaNeural",
            Voice::FilPhAngeloNeural => "fil-PH-AngeloNeural",
            Voice::FrBeCharlineNeural => "fr-BE-CharlineNeural",
            Voice::FrCaSylvieNeural => "fr-CA-SylvieNeural",
            Voice::FrChArianeNeural => "fr-CH-ArianeNeural",
            Voice::FrFrDeniseNeural => "fr-FR-DeniseNeural",
            Voice::GaIeColmNeural => "ga-IE-ColmNeural",
            Voice::GlEsRoiNeural => "gl-ES-RoiNeural",
            Voice::GuInDhwaniNeural => "gu-IN-DhwaniNeural",
            Voice::HeIlAvriNeural => "he-IL-AvriNeural",
            Voice::HiInMadhurNeural => "hi-IN-MadhurNeural",
            Voice::HrHrGabrijelaNeural => "hr-HR-GabrijelaNeural",
            Voice::HuHuNoemiNeural => "hu-HU-NoemiNeural",
            Voice::IdIdArdiNeural => "id-ID-ArdiNeural",
            Voice::IsIsGudrunNeural => "is-IS-GudrunNeural",
            Voice::ItItIsabellaNeural => "it-IT-IsabellaNeural",
            Voice::JaJpNanamiNeural => "ja-JP-NanamiNeural",
            Voice::JvIdDimasNeural => "jv-ID-DimasNeural",
            Voice::KkKzAigulNeural => "kk-KZ-AigulNeural",
            Voice::KmKhPisethNeural => "km-KH-PisethNeural",
            Voice::KnInGaganNeural => "kn-IN-GaganNeural",
            Voice::KoKrSunHiNeural => "ko-KR-SunHiNeural",
            Voice::LoLaChanthavongNeural => "lo-LA-ChanthavongNeural",
            Voice::LtLtLeonasNeural => "lt-LT-LeonasNeural",
            Voice::LvLvEveritaNeural => "lv-LV-EveritaNeural",
            Voice::MkMkAleksandarNeural => "mk-MK-AleksandarNeural",
            Voice::MlInMidhunNeural => "ml-IN-MidhunNeural",
            Voice::MrInAarohiNeural => "mr-IN-AarohiNeural",
            Voice::MsMyOsmanNeural => "ms-MY-OsmanNeural",
            Voice::MtMtGraceNeural => "mt-MT-GraceNeural",
            Voice::MyMmNilarNeural => "my-MM-NilarNeural",
            Voice::NbNoPernilleNeural => "nb-NO-PernilleNeural",
            Voice::NlBeArnaudNeural => "nl-BE-ArnaudNeural",
            Voice::NlNlColetteNeural => "nl-NL-ColetteNeural",
            Voice::PlPlAgnieszkaNeural => "pl-PL-AgnieszkaNeural",
            Voice::PsAfGulNawazNeural => "ps-AF-GulNawazNeural",
            Voice::PtBrFranciscaNeural => "pt-BR-FranciscaNeural",
            Voice::PtPtDuarteNeural => "pt-PT-DuarteNeural",
            Voice::RoRoAlinaNeural => "ro-RO-AlinaNeural",
            Voice::RuRuSvetlanaNeural => "ru-RU-SvetlanaNeural",
            Voice::SiLkSameeraNeural => "si-LK-SameeraNeural",
            Voice::SkSkLukasNeural => "sk-SK-LukasNeural",
            Voice::SlSiPetraNeural => "sl-SI-PetraNeural",
            Voice::SoSoMuuseNeural => "so-SO-MuuseNeural",
            Voice::SrRsNicholasNeural => "sr-RS-NicholasNeural",
            Voice::SuIdJajangNeural => "su-ID-JajangNeural",
            Voice::SvSeSofieNeural => "sv-SE-SofieNeural",
            Voice::SwKeRafikiNeural => "sw-KE-RafikiNeural",
            Voice::SwTzDaudiNeural => "sw-TZ-DaudiNeural",
            Voice::TaInPallaviNeural => "ta-IN-PallaviNeural",
            Voice::TaLkKumarNeural => "ta-LK-KumarNeural",
            Voice::TaSgAnbuNeural => "ta-SG-AnbuNeural",
            Voice::TeInMohanNeural => "te-IN-MohanNeural",
            Voice::ThThPremwadeeNeural => "th-TH-PremwadeeNeural",
            Voice::TrTrAhmetNeural => "tr-TR-AhmetNeural",
            Voice::UkUaOstapNeural => "uk-UA-OstapNeural",
            Voice::UrInGulNeural => "ur-IN-GulNeural",
            Voice::UrPkAsadNeural => "ur-PK-AsadNeural",
            Voice::UzUzMadinaNeural => "uz-UZ-MadinaNeural",
            Voice::ViVnHoaiMyNeural => "vi-VN-HoaiMyNeural",
            Voice::ZhCnXiaoxiaoNeural => "zh-CN-XiaoxiaoNeural",
            Voice::ZhHkHiuMaanNeural => "zh-HK-HiuMaanNeural",
            Voice::ZhTwHsiaoChenNeural => "zh-TW-HsiaoChenNeural",
            Voice::ZuZaThandoNeural => "zu-ZA-ThandoNeural",
            Voice::Specific(s) => s,
        }
    }
}