/// OOXML format schemas, content types, namespaces, and root tags.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OoxmlFormatKind {
    Docx,
    Docm,
    Xlsx,
    Xlsm,
    Pptx,
    Pptm,
}

impl OoxmlFormatKind {
    pub fn format_id(&self) -> &'static str {
        match self {
            Self::Docx => "docx",
            Self::Docm => "docm",
            Self::Xlsx => "xlsx",
            Self::Xlsm => "xlsm",
            Self::Pptx => "pptx",
            Self::Pptm => "pptm",
        }
    }

    pub fn recommended_extension(&self) -> &'static str {
        self.format_id()
    }

    pub fn evidence_code(&self) -> &'static str {
        match self {
            Self::Docx => "ooxml.word_main_part",
            Self::Docm => "ooxml.word_macro_main_part",
            Self::Xlsx => "ooxml.spreadsheet_main_part",
            Self::Xlsm => "ooxml.spreadsheet_macro_main_part",
            Self::Pptx => "ooxml.presentation_main_part",
            Self::Pptm => "ooxml.presentation_macro_main_part",
        }
    }

    pub fn expected_root_local(&self) -> &'static [u8] {
        match self {
            Self::Docx | Self::Docm => b"document",
            Self::Xlsx | Self::Xlsm => b"workbook",
            Self::Pptx | Self::Pptm => b"presentation",
        }
    }

    pub fn matches_namespace(&self, ns: &[u8]) -> bool {
        match self {
            Self::Docx | Self::Docm => {
                ns == b"http://schemas.openxmlformats.org/wordprocessingml/2006/main"
                    || ns == b"http://purl.oclc.org/ooxml/wordprocessingml/main"
            }
            Self::Xlsx | Self::Xlsm => {
                ns == b"http://schemas.openxmlformats.org/spreadsheetml/2006/main"
                    || ns == b"http://purl.oclc.org/ooxml/spreadsheetml/main"
            }
            Self::Pptx | Self::Pptm => {
                ns == b"http://schemas.openxmlformats.org/presentationml/2006/main"
                    || ns == b"http://purl.oclc.org/ooxml/presentationml/main"
            }
        }
    }
}

pub fn classify_ooxml_content_type(ct: &str) -> Option<OoxmlFormatKind> {
    match ct {
        // Word processing
        "application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"
        | "application/vnd.openxmlformats-officedocument.wordprocessingml.template.main+xml" => {
            Some(OoxmlFormatKind::Docx)
        }
        "application/vnd.ms-word.document.macroEnabled.main+xml"
        | "application/vnd.ms-word.template.macroEnabledTemplate.main+xml" => {
            Some(OoxmlFormatKind::Docm)
        }

        // Spreadsheet
        "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet.main+xml"
        | "application/vnd.openxmlformats-officedocument.spreadsheetml.template.main+xml" => {
            Some(OoxmlFormatKind::Xlsx)
        }
        "application/vnd.ms-excel.sheet.macroEnabled.main+xml"
        | "application/vnd.ms-excel.template.macroEnabled.main+xml" => Some(OoxmlFormatKind::Xlsm),

        // Presentation
        "application/vnd.openxmlformats-officedocument.presentationml.presentation.main+xml"
        | "application/vnd.openxmlformats-officedocument.presentationml.template.main+xml"
        | "application/vnd.openxmlformats-officedocument.presentationml.slideshow.main+xml" => {
            Some(OoxmlFormatKind::Pptx)
        }
        "application/vnd.ms-powerpoint.presentation.macroEnabled.main+xml"
        | "application/vnd.ms-powerpoint.template.macroEnabled.main+xml"
        | "application/vnd.ms-powerpoint.slideshow.macroEnabled.main+xml" => {
            Some(OoxmlFormatKind::Pptm)
        }

        _ => None,
    }
}
