# accessibility-rs

The Rust web accessibility engine.

```rs
// pass in html and css rules prior. If css rules are not passed in internal extraction is performed.
let audit = accessibility_rs::audit(&html, &css_rules, false);
```

## Features

1. Accurate web accessibility WCAG audits without a headless browser without missing a beat.
2. Re-creating layout trees to get element positions.
3. Ideal shapes for audits that scale.
4. Amazingly fast audits.

## Roadmap

1. All WCAGA-AAA Audits with rules mapped.
2. Next level performance.
3. Clean architecure maybe the code gets merged into a browser one day.
4. Improve Layout bounding accuracy to re-create leafs.

## About

This crate is actively being developed. Some of the initial code is set as stubs until the feature is complete.
We have three params for the initial audit `html`, `css_rules`, and `clip` to create bounding boxs. The reason we have `css_rules` as a param is if coming from a browser
you can get all the stylesheets from the dom of external sheets at once. This saves time in re-gathering links later.