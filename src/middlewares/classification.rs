//! jev-based classification middleware
use crate::middlewares::*;
use std::collections::HashMap;

pub struct ClassificationMiddleware {
    order: u32,
    jev: Arc<Jev>,
}

#[async_trait]
impl TaryMiddleware for ClassificationMiddleware {
    fn init(cfg: Arc<Config>, jev: Arc<Jev>) -> Result<Option<Arc<Self>>, Box<dyn Error>> {
        if let Some(cc) = &cfg.middlewares.classification
            && cc.enabled
        {
            Ok(Some(Arc::new(Self {
                order: cc.order,
                jev,
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
        use crate::content::ContentPriority as CP;

        if !self.jev.is_enabled() {
            return ct;
        }

        let mut state = JevRequest::state(
            format!(
                "{ctype}: {text}\\nn{due}{dest}",
                ctype = ct.content_type,
                due = ct
                    .due
                    .map(|d| format!(
                        "This item is due {rh} hours from now\n",
                        rh = (d - ct.date).num_milliseconds() as f64 / 3600000.0
                    ))
                    .unwrap_or_else(|| "".into()),
                dest = ct
                    .dest
                    .as_ref()
                    .map(|d| format!("This item has associated location: {d}\n"))
                    .unwrap_or_else(|| "".into()),
                text = ct.content
            )
            .as_str(),
        );

        if ct.content_priority.is_none() {
            state = state.add_question(
                "priority",
                JevQuestion::choice(
                    "What is the priority of this item?",
                    JevCriteria::Choice(HashMap::from(
                        [
                            (CP::Low, "Low priority: It is an item that is may not be time-sensitive."),
                            (CP::Normal, "Normal priority: this item is nice to have but may take a backseat to more important tasks."),
                            (CP::High, "High priority: this item shoud be done sooner than items of normal priority. It may include time-sensitive items or items that left incomplete may have adverse consequences."),
                        ]
                        .map(|x| (serde_json::to_string(&x.0).unwrap(), x.1.to_string())),
                    )),
                ),
            );
        }

        println!("{:?}", state);

        if state.questions.len() > 0 {
            let resp = self.jev.request(state).await.unwrap().answers; // TODO: safety
            println!("{:?}", resp);
            if let Some(ref cp) = resp.get(&"priority".to_string()) {
                ct.content_priority =
                    Some(serde_json::from_str(cp.choice.as_ref().unwrap().as_str()).unwrap());
            }
        }

        ct
    }
}
