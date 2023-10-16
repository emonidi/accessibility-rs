use crate::engine::rules::rule::{Rule, Validation};
use crate::engine::rules::techniques::Techniques;
use crate::engine::rules::utils::nodes::{
    get_unique_selector, has_alt, validate_empty_nodes, validate_missing_attr,
};
use crate::engine::rules::wcag_base::{Guideline, IssueType, Principle};
use crate::i18n::locales::get_message_i18n_str_raw;
use accessibility_scraper::{ElementRef, Selector};
use selectors::Element;
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::ops::Add;

lazy_static! {
    /// a list of rules that should be applied for WCAG2.0 A-AAA
    pub static ref RULES_A: BTreeMap<&'static str, Vec<Rule>> =
        vec![
            ("html", Vec::from([
                Rule::new(Techniques::H57.into(), IssueType::Error, Principle::Understandable, Guideline::Readable, "1", |nodes, _lang| {
                    let n = nodes[0].0;
                    Validation::new_issue(!n.attr("lang").unwrap_or_default().is_empty() || !n.attr("xml:lang").unwrap_or_default().is_empty(), "2")
                }),
                Rule::new(Techniques::H57.into(), IssueType::Error, Principle::Understandable, Guideline::Readable, "1", |nodes, _lang| {
                    let lang = nodes[0].0.attr("lang").unwrap_or_default();
                    let alphabetic = lang.chars().all(|x| x.is_alphabetic());
                    // <https://www.rfc-editor.org/rfc/bcp/bcp47.txt>
                    Validation::new_issue(if lang.len() > 3 {
                        let mut c = lang.chars();
                        let has_underscore = c.nth(2).unwrap_or_default() == '_' || lang.len() >= 4 && c.nth(1).unwrap_or_default() == '_';
                        alphabetic && has_underscore && lang.len() < 12
                    } else {
                        alphabetic && lang.len() < 12
                    }, "3.Lang")
                }),
                Rule::new(Techniques::H57.into(), IssueType::Error, Principle::Understandable, Guideline::Readable, "1", |nodes, _lang| {
                    let lang = nodes[0].0.attr("xml:lang").unwrap_or_default();
                    let alphabetic = lang.chars().all(|x| x == '_' || x.is_alphabetic());
                   // <https://www.rfc-editor.org/rfc/bcp/bcp47.txt>
                   Validation::new_issue(if lang.len() > 3 {
                        let mut c = lang.chars();
                        let has_underscore = c.nth(2).unwrap_or_default() == '_' || lang.len() >= 4 && c.nth(1).unwrap_or_default() == '_';
                        alphabetic && has_underscore && lang.len() < 12
                    } else {
                        alphabetic && lang.len() < 12
                    }, "3.XmlLang")
                }),
                Rule::new(Techniques::F77.into(), IssueType::Error, Principle::Robust, Guideline::Compatible, "1", |nodes, lang| {
                   let mut id_map: HashMap<&str, u8> = HashMap::new();
                   let mut valid = true;

                   for item in nodes {
                        let ele = item.0;
                        let tree = ele.tree();
                        for e in tree.nodes() {
                            match ElementRef::wrap(e) {
                                Some(element) => {
                                    match element.value().id() {
                                        Some(s) => {
                                            if id_map.contains_key(s) {
                                                let u = id_map.get(s);
                                                match u {
                                                    Some(u) => {
                                                        valid = false;
                                                        id_map.insert(s, u.add(1));
                                                    }
                                                    _ => ()
                                                }
                                            } else {
                                                id_map.insert(s, 1);
                                            }
                                        }
                                        _ => ()
                                    }
                                }
                                _ => (),
                            }
                        }
                   }

                   let duplicate_ids: Vec<_> = id_map.into_iter().filter_map(|(id, size)| if size >= 1 { Some("#".to_owned() + &id) } else { None }).collect();
                   let message = t!(&get_message_i18n_str_raw( &Guideline::Compatible, Techniques::F77.as_str(), "1", ""), locale = lang, id = duplicate_ids.join(","));

                   Validation::new(valid, "", duplicate_ids, message)
                }),
            ])),
            ("meta", Vec::from([
                Rule::new(Techniques::F40.into(), IssueType::Error, Principle::Operable, Guideline::EnoughTime, "1", |nodes, _lang| {
                    let mut valid = true;

                    for node in nodes {
                        let element = node.0;
                        let meta_refresh = element.attr("http-equiv").unwrap_or_default();
                        if meta_refresh == "refresh" {
                            let content = element.attr("content").unwrap_or_default();
                            if content.contains("url") {
                                valid = content.starts_with("0;");
                            }
                        }
                    }

                    Validation::new_issue(valid, "2")
                }),
                Rule::new(Techniques::F41.into(), IssueType::Error, Principle::Understandable, Guideline::EnoughTime, "1", |nodes, _lang| {
                    let mut valid = true;

                    for node in nodes {
                        let element = node.0;
                        let meta_refresh = element.attr("http-equiv").unwrap_or_default();
                        if meta_refresh == "refresh" {
                            let content = element.attr("content").unwrap_or_default();
                            if !content.is_empty() {
                                valid = content == "0";
                            }
                        }
                    }

                    Validation::new_issue(valid, "2")
                }),
            ])),
            ("title", Vec::from([
                Rule::new(Techniques::H25.into(), IssueType::Error, Principle::Operable, Guideline::Navigable, "1", |nodes, _lang| {
                    Validation::new_issue(!nodes.is_empty(), "1.NoTitleEl")
                }),
                Rule::new(Techniques::H25.into(), IssueType::Error, Principle::Operable, Guideline::Navigable, "1", |nodes, _lang| {
                    Validation::new_issue(nodes.is_empty() || nodes[0].0.html().is_empty(), "1.EmptyTitle")
                }),
            ])),
            ("blink", Vec::from([
                Rule::new(Techniques::F47.into(), IssueType::Error, Principle::Operable, Guideline::EnoughTime, "2", |nodes, _lang| {
                    Validation::new_issue(nodes.is_empty(), "")
                }),
            ])),
            ("iframe", Vec::from([
                Rule::new(Techniques::H64.into(), IssueType::Error, Principle::Operable, Guideline::Navigable, "1", |nodes, _lang| {
                    validate_missing_attr(nodes, "title", "1")
                }),
            ])),
            ("frame", Vec::from([
                Rule::new(Techniques::H64.into(), IssueType::Error, Principle::Operable, Guideline::Navigable, "1", |nodes, _lang| {
                    validate_missing_attr(nodes, "title", "1")
                }),
            ])),
            ("form", Vec::from([
                Rule::new(Techniques::H32.into(), IssueType::Error, Principle::Operable, Guideline::Predictable, "2", |nodes, _lang| {
                    let mut valid = false;
                    let mut elements = Vec::new();
                    let selector = unsafe { Selector::parse("button[type=submit]").unwrap_unchecked() };

                    for ele in nodes {
                        let ele = ele.0;
                        let e = ele.select(&selector);
                        let c = e.count();

                       if c == 1 {
                            valid = true;
                       } else {
                            valid = false;
                            elements.push(get_unique_selector(&ele))
                        }
                    }

                    Validation::new(valid, "2", elements, Default::default())
                }),
                Rule::new(Techniques::H36.into(), IssueType::Error, Principle::Perceivable, Guideline::TextAlternatives, "1", |nodes, _lang| {
                    let mut valid = false;
                    let mut elements = Vec::new();
                    let selector = unsafe { Selector::parse("input[type=image][name=submit]").unwrap_unchecked() };

                    for ele in nodes {
                        let ele = ele.0;
                        let mut e = ele.select(&selector);

                        while let Some(el) = e.next() {
                            let alt = has_alt(el);
                            if !alt {
                                elements.push(get_unique_selector(&ele))
                            }
                            valid = alt;
                        }
                    }

                    Validation::new(valid, "", elements, Default::default())
                }),
            ])),
            ("a", Vec::from([
                Rule::new(Techniques::H30.into(), IssueType::Error, Principle::Perceivable, Guideline::TextAlternatives, "1", |nodes, _lang| {
                    let mut valid = true;
                    let selector = unsafe { Selector::parse("img").unwrap_unchecked() };
                    let mut elements = Vec::new();

                    for ele in nodes {
                        let ele = ele.0;
                        let mut e = ele.select(&selector);

                        while let Some(el) = e.next() {
                            let alt = has_alt(el);
                            if !alt {
                                elements.push(get_unique_selector(&ele))
                            }
                            valid = alt;
                        }
                    }

                    Validation::new(valid, "2", elements, Default::default())
                }),
                Rule::new(Techniques::H91.into(), IssueType::Error, Principle::Robust, Guideline::Compatible, "2", |nodes, _lang| {
                    let mut valid = true;
                    let mut elements = Vec::new();

                    for ele in nodes {
                        let ele = ele.0;
                        match ele.attr("href") {
                            Some(_) => {
                                let empty = ele.inner_html().trim().is_empty();
                                if empty {
                                    elements.push(get_unique_selector(&ele))
                                }
                                valid = !empty
                            }
                            _ => ()
                        }
                    }
                    Validation::new(valid, "A.NoContent", elements, Default::default())
                }),
                Rule::new(Techniques::H91.into(), IssueType::Error, Principle::Robust, Guideline::Compatible, "2", |nodes, _lang| {
                    let mut valid = true;
                    let mut elements = Vec::new();
                    for ele in nodes {
                        let ele = ele.0;
                        let v = !ele.is_empty() || ele.has_attribute("id") || ele.has_attribute("href");
                        if !v {
                            elements.push(get_unique_selector(&ele))
                        }
                        valid = v;
                    }
                    Validation::new(valid, "A.EmptyNoId", elements, Default::default())
                }),
            ])),
            ("img", Vec::from([
                Rule::new(Techniques::H37.into(), IssueType::Error, Principle::Perceivable, Guideline::TextAlternatives, "1", |nodes, _lang| {
                    let mut valid = true;
                    let mut elements = Vec::new();

                    for ele in nodes {
                        let ele = ele.0;
                        let alt = has_alt(ele);
                        if !alt {
                            elements.push(get_unique_selector(&ele))
                        }
                        valid = alt;
                    }

                    Validation::new(valid, "", elements, Default::default())
                }),
            ])),
            ("h1", Vec::from([
                Rule::new(Techniques::H42.into(), IssueType::Error, Principle::Perceivable, Guideline::Adaptable, "1", |nodes, _lang| {
                    validate_empty_nodes(nodes, "2")
                }),
            ])),
            ("h2", Vec::from([
                Rule::new(Techniques::H42.into(), IssueType::Error, Principle::Perceivable, Guideline::Adaptable, "1", |nodes, _lang| {
                    validate_empty_nodes(nodes, "2")
                }),
            ])),
            ("h3", Vec::from([
                Rule::new(Techniques::H42.into(), IssueType::Error, Principle::Perceivable, Guideline::Adaptable, "1", |nodes, _lang| {
                    validate_empty_nodes(nodes, "2")
                }),
            ])),
            ("h4", Vec::from([
                Rule::new(Techniques::H42.into(), IssueType::Error, Principle::Perceivable, Guideline::Adaptable, "1", |nodes, _lang| {
                    validate_empty_nodes(nodes, "2")
                }),
            ])),
            ("h5", Vec::from([
                Rule::new(Techniques::H42.into(), IssueType::Error, Principle::Perceivable, Guideline::Adaptable, "1", |nodes, _lang| {
                    validate_empty_nodes(nodes, "2")
                }),
            ])),
            ("h6", Vec::from([
                Rule::new(Techniques::H42.into(), IssueType::Error, Principle::Perceivable, Guideline::Adaptable, "1", |nodes, _lang| {
                    validate_empty_nodes(nodes, "2")
                }),
            ])),
        ]
        .into_iter()
        .collect();
}
