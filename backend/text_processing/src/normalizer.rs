use unicode_normalization::UnicodeNormalization;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChineseVariant {
    Preserve,
    Simplified,
    Traditional,
}

pub fn normalize(text: &str) -> String {
    normalize_with_variant(text, ChineseVariant::Preserve)
}

pub fn normalize_with_variant(text: &str, variant: ChineseVariant) -> String {
    let nfc: String = text.nfc().collect();
    let collapsed = nfc.split_whitespace().collect::<Vec<_>>().join(" ");

    match variant {
        ChineseVariant::Preserve => collapsed,
        ChineseVariant::Simplified => to_simplified(&collapsed),
        ChineseVariant::Traditional => to_traditional(&collapsed),
    }
}

pub fn to_simplified(text: &str) -> String {
    text.chars().map(simplified_char).collect()
}

pub fn to_traditional(text: &str) -> String {
    text.chars().map(traditional_char).collect()
}

fn simplified_char(character: char) -> char {
    match character {
        '電' => '电',
        '話' => '话',
        '給' => '给',
        '學' => '学',
        '習' => '习',
        '語' => '语',
        '體' => '体',
        '國' => '国',
        '來' => '来',
        '這' => '这',
        '個' => '个',
        '為' => '为',
        '與' => '与',
        '開' => '开',
        '關' => '关',
        '時' => '时',
        '間' => '间',
        '說' => '说',
        '聽' => '听',
        '讀' => '读',
        '寫' => '写',
        _ => character,
    }
}

fn traditional_char(character: char) -> char {
    match character {
        '电' => '電',
        '话' => '話',
        '给' => '給',
        '学' => '學',
        '习' => '習',
        '语' => '語',
        '体' => '體',
        '国' => '國',
        '来' => '來',
        '这' => '這',
        '个' => '個',
        '为' => '為',
        '与' => '與',
        '开' => '開',
        '关' => '關',
        '时' => '時',
        '间' => '間',
        '说' => '說',
        '听' => '聽',
        '读' => '讀',
        '写' => '寫',
        _ => character,
    }
}
