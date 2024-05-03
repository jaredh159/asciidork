// use test_utils::*;

// mod helpers;

// test_eval!(
//   basic_table,
//   adoc! {r#"
//     |===
//     |a | b
//     |c | d
//     |===
//   "#},
//   html! {r#"
//     <table class="tableblock frame-all grid-all stretch">
//       <colgroup>
//         <col style="width: 50%;">
//         <col style="width: 50%;">
//       </colgroup>
//       <tbody>
//         <tr>
//           <td class="tableblock halign-left valign-top">
//             <p class="tableblock">a</p>
//           </td>
//           <td class="tableblock halign-left valign-top">
//             <p class="tableblock">b</p>
//           </td>
//         </tr>
//         <tr>
//           <td class="tableblock halign-left valign-top">
//             <p class="tableblock">c</p>
//           </td>
//           <td class="tableblock halign-left valign-top">
//             <p class="tableblock">d</p>
//           </td>
//         </tr>
//       </tbody>
//     </table>
//   "#}
// );
