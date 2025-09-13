use crate::adapters::location::GeoLocation;

impl GeoLocation {
    pub fn from_name_and_campus(location: &str, _campus: &str) -> Option<Self> {
        if location.contains("仙Ⅰ") {
            // https://maps.apple.com/?ll=32.111571,118.959550&q=%E5%8D%97%E4%BA%AC%E4%BF%A1%E6%81%AF%E8%81%8C%E4%B8%9A%E6%8A%80%E6%9C%AF%E5%AD%A6%E9%99%A2&spn=0.000996,0.001228&t=m
            GeoLocation::new(32.111571, 118.959550).into()
        } else if location.contains("仙Ⅱ") {
            // https://maps.apple.com/?ll=32.112285,118.959041&q=%E5%8D%97%E4%BA%AC%E4%BF%A1%E6%81%AF%E8%81%8C%E4%B8%9A%E6%8A%80%E6%9C%AF%E5%AD%A6%E9%99%A2&spn=0.000996,0.001228&t=m
            GeoLocation::new(32.112285, 118.959041).into()
        } else if location.contains("方肇周") {
            // https://maps.apple.com/?ll=32.112693,118.956220&q=%E5%8D%97%E4%BA%AC%E4%BF%A1%E6%81%AF%E8%81%8C%E4%B8%9A%E6%8A%80%E6%9C%AF%E5%AD%A6%E9%99%A2&spn=0.001458,0.001795&t=m
            GeoLocation::new(32.112693, 118.956220).into()
        } else if location.contains("基础实验楼乙") {
            // https://maps.apple.com/?ll=32.110261,118.957089&q=%E5%8D%97%E4%BA%AC%E4%BF%A1%E6%81%AF%E8%81%8C%E4%B8%9A%E6%8A%80%E6%9C%AF%E5%AD%A6%E9%99%A2&spn=0.001135,0.001397&t=m
            GeoLocation::new(32.110261, 118.957089).into()
        } else if location.contains("基础实验楼丙") {
            // https://maps.apple.com/?ll=32.110409,118.958241&q=%E5%8D%97%E4%BA%AC%E4%BF%A1%E6%81%AF%E8%81%8C%E4%B8%9A%E6%8A%80%E6%9C%AF%E5%AD%A6%E9%99%A2&spn=0.000828,0.001020&t=m
            GeoLocation::new(32.110409, 118.958241).into()
        } else if location.contains("基础实验楼甲") {
            // https://maps.apple.com/?ll=32.110065,118.955857&q=%E5%8D%97%E4%BA%AC%E4%BF%A1%E6%81%AF%E8%81%8C%E4%B8%9A%E6%8A%80%E6%9C%AF%E5%AD%A6%E9%99%A2&spn=0.000830,0.001022&t=m
            GeoLocation::new(32.110065, 118.955857).into()
        } else if location.contains("逸") {
            // https://maps.apple.com/?ll=32.110602,118.959645&q=%E5%8D%97%E4%BA%AC%E4%BF%A1%E6%81%AF%E8%81%8C%E4%B8%9A%E6%8A%80%E6%9C%AF%E5%AD%A6%E9%99%A2&spn=0.001369,0.001685&t=m
            GeoLocation::new(32.110602, 118.959645).into()
        } else if location.contains("化学楼") {
            // https://maps.apple.com/?ll=32.118459,118.952461&q=%E5%8D%97%E4%BA%AC%E4%BF%A1%E6%81%AF%E8%81%8C%E4%B8%9A%E6%8A%80%E6%9C%AF%E5%AD%A6%E9%99%A2&spn=0.001545,0.001942&t=m
            GeoLocation::new(32.118459, 118.952461).into()
        } else if location.contains("环科楼") {
            // https://maps.apple.com/?ll=32.117099,118.953059&q=%E5%8D%97%E4%BA%AC%E4%BF%A1%E6%81%AF%E8%81%8C%E4%B8%9A%E6%8A%80%E6%9C%AF%E5%AD%A6%E9%99%A2&spn=0.001539,0.001935&t=m
            GeoLocation::new(32.117099, 118.953059).into()
        } else if location.contains("大气楼") {
            // https://maps.apple.com/?ll=32.117680,118.955216&q=%E5%8D%97%E4%BA%AC%E4%BF%A1%E6%81%AF%E8%81%8C%E4%B8%9A%E6%8A%80%E6%9C%AF%E5%AD%A6%E9%99%A2&spn=0.000931,0.001170&t=m
            GeoLocation::new(32.117680, 118.955216).into()
        } else if location.contains("地海楼") {
            // https://maps.apple.com/?ll=32.112540,118.961573&q=%E5%8D%97%E4%BA%AC%E4%BF%A1%E6%81%AF%E8%81%8C%E4%B8%9A%E6%8A%80%E6%9C%AF%E5%AD%A6%E9%99%A2&spn=0.001280,0.001585&t=m
            GeoLocation::new(32.112540, 118.961573).into()
        } else if location.contains("地科楼") {
            // https://maps.apple.com/?ll=32.111781,118.961577&q=%E5%8D%97%E4%BA%AC%E4%BF%A1%E6%81%AF%E8%81%8C%E4%B8%9A%E6%8A%80%E6%9C%AF%E5%AD%A6%E9%99%A2&spn=0.001130,0.001398&t=m
            GeoLocation::new(32.111781, 118.961577).into()
        } else if location.contains("电子楼") {
            // https://maps.apple.com/?ll=32.110843,118.961881&q=%E5%8D%97%E4%BA%AC%E4%BF%A1%E6%81%AF%E8%81%8C%E4%B8%9A%E6%8A%80%E6%9C%AF%E5%AD%A6%E9%99%A2&spn=0.001192,0.001475&t=m
            GeoLocation::new(32.110843, 118.961881).into()
        } else if location.contains("计科楼") {
            // https://maps.apple.com/?ll=32.111006,118.963210&q=%E5%8D%97%E4%BA%AC%E4%BF%A1%E6%81%AF%E8%81%8C%E4%B8%9A%E6%8A%80%E6%9C%AF%E5%AD%A6%E9%99%A2&spn=0.000965,0.001194&t=m
            GeoLocation::new(32.111006, 118.963210).into()
        } else if location.contains("行政楼") {
            // https://maps.apple.com/?ll=32.112017,118.963088&q=%E5%8D%97%E4%BA%AC%E4%BF%A1%E6%81%AF%E8%81%8C%E4%B8%9A%E6%8A%80%E6%9C%AF%E5%AD%A6%E9%99%A2&spn=0.000755,0.000935&t=m
            GeoLocation::new(32.112017, 118.963088).into()
        } else if location.contains("天文楼") {
            // https://maps.apple.com/?ll=32.125405,118.959940&q=%E5%8D%97%E4%BA%AC%E4%BF%A1%E6%81%AF%E8%81%8C%E4%B8%9A%E6%8A%80%E6%9C%AF%E5%AD%A6%E9%99%A2&spn=0.000599,0.000753&t=m
            GeoLocation::new(32.125405, 118.959940).into()
        } else if location.contains("众创空间") {
            // https://maps.apple.com/?ll=32.122708,118.952153&q=%E5%8D%97%E4%BA%AC%E4%BF%A1%E6%81%AF%E8%81%8C%E4%B8%9A%E6%8A%80%E6%9C%AF%E5%AD%A6%E9%99%A2&spn=0.000669,0.000841&t=m
            GeoLocation::new(32.122708, 118.952153).into()
        } else if location.contains("社会学院") {
            // https://maps.apple.com/?ll=32.118196,118.959968&q=%E5%8D%97%E4%BA%AC%E4%BF%A1%E6%81%AF%E8%81%8C%E4%B8%9A%E6%8A%80%E6%9C%AF%E5%AD%A6%E9%99%A2&spn=0.000778,0.000978&t=m
            GeoLocation::new(32.118196, 118.959968).into()
        } else if location.contains("历史学院") {
            // https://maps.apple.com/?ll=32.118890,118.959353&q=%E5%8D%97%E4%BA%AC%E4%BF%A1%E6%81%AF%E8%81%8C%E4%B8%9A%E6%8A%80%E6%9C%AF%E5%AD%A6%E9%99%A2&spn=0.000884,0.001111&t=m
            GeoLocation::new(32.118890, 118.959353).into()
        } else if location.contains("政管学院") {
            // https://maps.apple.com/?ll=32.117351,118.959900&q=%E5%8D%97%E4%BA%AC%E4%BF%A1%E6%81%AF%E8%81%8C%E4%B8%9A%E6%8A%80%E6%9C%AF%E5%AD%A6%E9%99%A2&spn=0.000749,0.000942&t=m
            GeoLocation::new(32.117351, 118.959900).into()
        } else if location.contains("生科楼") {
            // https://maps.apple.com/?ll=32.119247,118.954984&q=%E5%8D%97%E4%BA%AC%E4%BF%A1%E6%81%AF%E8%81%8C%E4%B8%9A%E6%8A%80%E6%9C%AF%E5%AD%A6%E9%99%A2&spn=0.001296,0.001629&t=m
            GeoLocation::new(32.119247, 118.954984).into()
        } else if location.contains("医学楼") {
            // https://maps.apple.com/?ll=32.119974,118.954473&q=%E5%8D%97%E4%BA%AC%E4%BF%A1%E6%81%AF%E8%81%8C%E4%B8%9A%E6%8A%80%E6%9C%AF%E5%AD%A6%E9%99%A2&spn=0.000528,0.000664&t=m
            GeoLocation::new(32.119974, 118.954473).into()
        } else if location.contains("现工院楼") {
            // https://maps.apple.com/?ll=32.121247,118.955225&q=%E5%8D%97%E4%BA%AC%E4%BF%A1%E6%81%AF%E8%81%8C%E4%B8%9A%E6%8A%80%E6%9C%AF%E5%AD%A6%E9%99%A2&spn=0.001590,0.001999&t=m
            GeoLocation::new(32.121247, 118.955225).into()
        } else if location.contains("四组团") {
            // https://maps.apple.com/?ll=32.121168,118.951608&q=Qixia%20%E2%80%94%20Nanjing&spn=0.000428,0.000776&t=m
            GeoLocation::new(32.121168, 118.951608).into()
        } else {
            None
        }
    }
}
