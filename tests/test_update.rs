use log::*;
use sauron::{
    html::{
        attributes::*,
        events::*,
        *,
    },
    mt_dom::patch::*,
    Patch,
    *,
};
use std::{
    cell::RefCell,
    rc::Rc,
};
use test_fixtures::simple_program;
use wasm_bindgen_test::*;
use web_sys::InputEvent;

mod test_fixtures;

wasm_bindgen_test_configure!(run_in_browser);

/// this is an inefficient patch since the blank lines are matches
/// causing the next good match to skip due to the there has been
///  a previous match with bigger node_idx
///  The solution is therefore to not put key to elements that
///  are meant to be discarded  and can easily be construcated
#[wasm_bindgen_test]
#[test]
fn test1() {
    let current_dom: Node<()> = node!(
    <div class="app">
       <h1>"Lines"</h1>
       <div>
          <div class="grid__wrapper">
             <div class="grid grid__number_wide1">
                <div class="grid__number__line" key="623356695095054844">
                   <div class="grid__number">"0"</div>
                   <div class="grid__line">
                      <div>"C"</div>
                      <div>"J"</div>
                      <div>"K"</div>
                      <div>"\n"</div>
                   </div>
                </div>
                <div class="grid__number__line" key="4638962052468762037">
                   <div class="grid__number">"1"</div>
                   <div class="grid__line">
                      <div>"\n"</div>
                   </div>
                </div>
                <div class="grid__number__line" key="9824372840226575955">
                   <div class="grid__number">"2"</div>
                   <div class="grid__line">
                      <div>"T"</div>
                      <div>"h"</div>
                      <div>"e"</div>
                      <div>"\n"</div>
                   </div>
                </div>
             </div>
             <div class="grid__status">"line: 0, column: 0"</div>
          </div>
       </div>
    </div>
    );

    let target_dom: Node<()> = node!(
     <div class="app">
        <h1>"Lines"</h1>
        <div>
           <div class="grid__wrapper">
              <div class="grid grid__number_wide1">
                 <div class="grid__number__line" key="4638962052468762037">
                    <div class="grid__number">"0"</div>
                    <div class="grid__line">
                       <div>"\n"</div>
                    </div>
                 </div>
                 <div class="grid__number__line" key="623356695095054844">
                    <div class="grid__number">"1"</div>
                    <div class="grid__line">
                       <div>"C"</div>
                       <div>"J"</div>
                       <div>"K"</div>
                       <div>"\n"</div>
                    </div>
                 </div>
                 <div class="grid__number__line" key="4638962052468762037">
                    <div class="grid__number">"2"</div>
                    <div class="grid__line">
                       <div>"\n"</div>
                    </div>
                 </div>
                 <div class="grid__number__line" key="9824372840226575955">
                    <div class="grid__number">"3"</div>
                    <div class="grid__line">
                       <div>"T"</div>
                       <div>"h"</div>
                       <div>"e"</div>
                       <div>"\n"</div>
                    </div>
                 </div>
              </div>
              <div class="grid__status">"line: 1, column: 0"</div>
           </div>
        </div>
     </div>
    );

    let patch = diff(&current_dom, &target_dom);
    dbg!(&patch);
    assert_eq!(
        patch,
        vec![
            ChangeText::new(20, "1", "0").into(),
            ChangeText::new(26, "2", "3").into(),
            InsertNode::new(
                Some(&"div"),
                12,
                &div(
                    vec![
                        class("grid__number__line"),
                        key("623356695095054844")
                    ],
                    vec![
                        div(vec![class("grid__number")], vec![text(1)]),
                        div(
                            vec![class("grid__line")],
                            vec![
                                div(vec![], vec![text("C")]),
                                div(vec![], vec![text("J")]),
                                div(vec![], vec![text("K")]),
                                div(vec![], vec![text("\n")]),
                            ]
                        ),
                    ]
                ),
            )
            .into(),
            InsertNode::new(
                Some(&"div"),
                13,
                &div(
                    vec![
                        class("grid__number__line"),
                        key("4638962052468762037")
                    ],
                    vec![
                        div(vec![class("grid__number")], vec![text(2)]),
                        div(
                            vec![class("grid__line")],
                            vec![div(vec![], vec![text("\n")]),]
                        ),
                    ]
                )
            )
            .into(),
            RemoveNode::new(Some(&"div"), 6).into(),
            ChangeText::new(37, "line: 0, column: 0", "line: 1, column: 0")
                .into(),
        ]
    );
}