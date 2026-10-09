use crate::middlewares::*;
use regex::Regex;

pub struct RegexMiddleware {
    regexes: Vec<(Regex, String)>,
    order: u32,
}

#[async_trait]
impl TaryMiddleware for RegexMiddleware {
    fn init(cfg: Arc<Config>, _jev: Arc<Jev>) -> Result<Option<Arc<Self>>, Box<dyn Error>> {
        if let Some(rc) = &cfg.middlewares.regex
            && rc.enabled
        {
            let mut rv = Vec::<(Regex, String)>::with_capacity(rc.regexes.len());
            for (re_s, rp_s) in rc.regexes.iter() {
                let re = Regex::new(re_s)?;
                rv.push((re, rp_s.to_string()));
            }
            Ok(Some(Arc::new(Self {
                order: rc.order,
                regexes: rv,
            })))
        } else {
            Ok(None)
        }
    }

    fn order(&self) -> u32 {
        self.order
    }

    fn has_transform(&self) -> bool {
        true
    }

    async fn transform(&self, mut ct: Content) -> Content {
        for (re, rp) in self.regexes.iter() {
            ct.content = re.replace_all(&ct.content, rp).to_string();
        }
        ct
    }
}
