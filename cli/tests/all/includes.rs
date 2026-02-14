use crate::helpers::*;
use test_utils::*;

#[cfg(unix)]
#[test]
fn test_cli_app_single_include() {
  let stdout = run_file(
    &["--embedded", "--strict", "--safe-mode", "unsafe"],
    "tests/all/fixtures/gen/a.adoc",
  );
  expect_eq!(
    stdout.trim(),
    html! {r#"
      <div class="paragraph">
        <p>docdir: {cwd}/tests/all/fixtures/gen</p>
      </div>
      <div class="paragraph">
        <p>f: <em>fixtures/gen/a.adoc</em></p>
      </div>
      <div class="paragraph">
        <p>docdir: {cwd}/tests/all/fixtures/gen</p>
      </div>
      <div class="paragraph">
        <p>f: <em>fixtures/gen/b.adoc</em></p>
      </div>
    "#}
    .replace("{cwd}", &cwd())
  );
}

#[test]
fn test_cli_include_case_fail_strict() {
  let stderr = run_expecting_err(
    &["--embedded", "--strict", "--safe-mode", "unsafe"],
    "tests/all/fixtures/gen/case-fail.adoc",
  );

  #[cfg(any(target_os = "windows", target_os = "macos"))]
  expect_eq!(
    stderr.trim(),
    adoc! {r#"
      --> case-fail.adoc:1:10
        |
      1 | include::sub/inNER.adoc[]
        |          ^^^^^^^^^^^^^^ Include error: Case mismatch in file path. Maybe you meant to include `inner.adoc`?

      Error: "Parse error""#}
  );

  #[cfg(target_os = "linux")]
  expect_eq!(
    stderr.trim(),
    adoc! {r#"
      --> case-fail.adoc:1:10
        |
      1 | include::sub/inNER.adoc[]
        |          ^^^^^^^^^^^^^^ Include error: File not found

      Error: "Parse error""#}
  );
}

#[cfg(unix)]
#[test]
fn test_relative_includes() {
  let stdout = run_file(
    &["--embedded", "--strict", "--safe-mode", "unsafe"],
    "tests/all/fixtures/gen/parent-include.adoc",
  );
  expect_eq!(
    stdout.trim(),
    html! {r#"
      <div class="paragraph"><p>first line of parent</p></div>
      <div class="paragraph"><p>first line of child</p></div>
      <div class="paragraph"><p>first line of grandchild</p></div>
      <div class="paragraph"><p>last line of grandchild</p></div>
      <div class="paragraph"><p>last line of child</p></div>
      <div class="paragraph"><p>last line of parent</p></div>
    "#}
  );
}

// run on linux (CI) only for speed in local dev
#[cfg(target_os = "linux")]
#[test]
fn test_remote_relative_includes() {
  let stdout = run_file(
    &[
      "--embedded",
      "--strict",
      "--safe-mode",
      "unsafe",
      "--attribute",
      "allow-uri-read",
    ],
    "tests/all/fixtures/remote-rel.adoc",
  );
  expect_eq!(
    stdout.trim(),
    html! {r#"
      <div class="paragraph"><p>first line of parent</p></div>
      <div class="paragraph"><p>first line of child</p></div>
      <div class="paragraph"><p>first line of grandchild</p></div>
      <div class="paragraph"><p>last line of grandchild</p></div>
      <div class="paragraph"><p>last line of child</p></div>
      <div class="paragraph"><p>last line of parent</p></div>
    "#}
  );
}

#[cfg(unix)]
#[test]
fn test_relative_nested_includes() {
  let stdout = run_file(
    &["--embedded", "--strict", "--safe-mode", "unsafe"],
    "tests/all/fixtures/gen/relative-include.adoc",
  );
  expect_eq!(
    stdout.trim(),
    html! {r#"
      <div class="paragraph"><p>first line of outer</p></div>
      <div class="paragraph"><p>first line of middle</p></div>
      <div class="paragraph"><p>only line of inner</p></div>
      <div class="paragraph"><p>last line of middle</p></div>
      <div class="paragraph"><p>last line of outer</p></div>
    "#}
  );
}

// run on linux (CI) only for speed in local dev
#[cfg(target_os = "linux")]
#[test]
fn test_url_includes() {
  let stdout = run_file(
    &[
      "--embedded",
      "--strict",
      "--safe-mode",
      "unsafe",
      "--attribute",
      "allow-uri-read",
    ],
    "tests/all/fixtures/remote.adoc",
  );
  expect_eq!(
    stdout.trim(),
    html! {r#"
      <div class="paragraph"><p>line 1</p></div>
      <div class="paragraph"><p>from <em>github</em></p></div>
    "#}
  );
}

// run on linux (CI) only for speed in local dev
#[cfg(target_os = "linux")]
#[test]
fn test_svg_uri() {
  let stdout = run_file(
    &[
      "--embedded",
      "--strict",
      "--safe-mode",
      "unsafe",
      "--attribute",
      "allow-uri-read",
    ],
    "tests/all/fixtures/remote-svg.adoc",
  );
  expect_eq!(
    stdout.trim(),
    html! {r#"
    <div class="imageblock">
      <div class="content">
        <svg width="200" xmlns="http://www.w3.org/2000/svg"><path d="M0 0"/></svg>
      </div>
    </div>
    <div class="imageblock">
      <div class="content">
        <img src="data:image/svg+xml;base64,PHN2ZyB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciPjxwYXRoIGQ9Ik0wIDAiLz48L3N2Zz4=" alt="mini">
      </div>
    </div>
    "#}
  );
}

#[test]
fn test_inline_svg_and_data_uri() {
  let stdout = run_file(
    &["--embedded", "--safe-mode", "safe"],
    "tests/all/fixtures/inline-svg.adoc",
  );
  expect_eq!(
    stdout.trim(),
    html! {r#"
    <div class="imageblock">
      <div class="content">
        <svg width="200" xmlns="http://www.w3.org/2000/svg"><path d="M0 0"/></svg>
      </div>
    </div>
    <div class="imageblock">
      <div class="content">
        <span class="alt">Panchito</span>
      </div>
    </div>
    <div class="imageblock">
      <div class="content">
        <span class="alt">restricted</span>
      </div>
    </div>
    <div class="imageblock">
      <div class="content">
        <svg height="150" xmlns="http://www.w3.org/2000/svg"><path d="M0 0"/></svg>
      </div>
    </div>
    <div class="imageblock">
      <div class="content">
        <img src="data:image/svg+xml;base64,PHN2ZyB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciPjxwYXRoIGQ9Ik0wIDAiLz48L3N2Zz4K" alt="mini">
      </div>
    </div>
    <div class="imageblock">
      <div class="content">
        <img src="data:image/svg+xml;base64," alt="restricted">
      </div>
    </div>
    <div class="admonitionblock tip">
      <table>
        <tr>
          <td class="icon"><img src="data:image/png;base64," alt="Tip"></td>
          <td class="content">a tip</td>
        </tr>
      </table>
    </div>
    <div class="admonitionblock note">
      <table>
        <tr>
          <td class="icon">
            <img src="data:image/gif;base64,R0lGODdhAQABAIABAAAAACEHbSwAAAAAAQABAAACAkwBADs=" alt="Note">
          </td>
          <td class="content">a note</td>
        </tr>
      </table>
    </div>
    <div class="admonitionblock warning">
      <table>
        <tr>
          <td class="icon">
            <img src="data:image/gif;base64,R0lGODdhAQABAIABAAAAAOYPwSwAAAAAAQABAAACAkwBADs=" alt="Warning">
          </td>
          <td class="content">Be careful!</td>
        </tr>
      </table>
    </div>
    <div class="admonitionblock warning">
      <table>
        <tr>
          <td class="icon"><img src="./images/icons/custom.gif" alt="Warning"></td>
          <td class="content">Be careful!</td>
        </tr>
      </table>
    </div>
    "#}
  );
}

#[cfg(unix)]
#[test]
fn test_cli_app_doc_attrs() {
  let stdout = run_file(
    &["--embedded", "--strict", "--safe-mode", "unsafe"],
    "tests/all/fixtures/gen/attrs.adoc",
  );
  expect_eq!(
    stdout.trim(),
    html! {r#"
      <div class="paragraph">
        <p>f: <em>fixtures/gen/attrs.adoc</em></p>
      </div>
      <div class="paragraph">
        <p>docdir: {cwd}/tests/all/fixtures/gen</p>
      </div>
      <div class="paragraph">
        <p>docfile: {cwd}/tests/all/fixtures/gen/attrs.adoc</p>
      </div>
      <div class="paragraph">
        <p>docfilesuffix: .adoc</p>
      </div>
      <div class="paragraph">
        <p>docname: attrs</p>
      </div>
    "#}
    .replace("{cwd}", &cwd())
  );
}

#[test]
fn test_cli_runs_on_windows() {
  let stdout = run_file(
    &["--embedded", "--strict", "--safe-mode", "unsafe"],
    "tests/all/fixtures/gen/gchild-include.adoc",
  );
  expect_eq!(
    stdout.trim(),
    html! {r#"
      <div class="paragraph">
        <p>first line of grandchild</p>
      </div>
      <div class="paragraph">
        <p>last line of grandchild</p>
      </div>
    "#}
    .replace("{cwd}", &cwd())
  );
}

#[test]
fn test_cli_doctype() {
  let stdout = run_file(&[], "tests/all/fixtures/book.adoc");
  assert!(stdout.contains("doctype: book"));
}

#[test]
fn test_secure_mode_blocks_document_icons_attr() {
  // In SECURE mode, document-defined :icons: should be BLOCKED (text label fallback)
  let stdout = run_input(
    &["--embedded", "--safe-mode", "secure"],
    adoc! {r#"
      :icons:
      :iconsdir: images/icons
      :icontype: gif

      NOTE: Icons set in document should be blocked in secure mode.
    "#},
  );
  expect_eq!(
    stdout.trim(),
    html! {r#"
      <div class="admonitionblock note">
        <table>
          <tr>
            <td class="icon"><div class="title">Note</div></td>
            <td class="content">Icons set in document should be blocked in secure mode.</td>
          </tr>
        </table>
      </div>
    "#}
  );
}

#[test]
fn test_secure_mode_allows_cli_icons_attr() {
  let stdout = run_input(
    &[
      "--embedded",
      "--safe-mode",
      "secure",
      "--attribute",
      "icons",
    ],
    adoc! {r#"
      :iconsdir: images/icons
      :icontype: gif
      :data-uri:

      NOTE: CLI icons works, but document data-uri is blocked.

      [WARNING,icon=custom]
      Custom icon also resolves via CLI-enabled icons.
    "#},
  );
  expect_eq!(
    stdout.trim(),
    html! {r#"
      <div class="admonitionblock note">
        <table>
          <tr>
            <td class="icon"><img src="images/icons/note.gif" alt="Note"></td>
            <td class="content">CLI icons works, but document data-uri is blocked.</td>
          </tr>
        </table>
      </div>
      <div class="admonitionblock warning">
        <table>
          <tr>
            <td class="icon"><img src="images/icons/custom.gif" alt="Warning"></td>
            <td class="content">Custom icon also resolves via CLI-enabled icons.</td>
          </tr>
        </table>
      </div>
    "#}
  );
}

#[test]
fn test_server_mode_masks_path_attributes() {
  let stdout = run_file(
    &["--embedded", "--safe-mode", "server"],
    "tests/all/fixtures/attr-masking.adoc",
  );
  expect_eq!(
    stdout.trim(),
    html! {r#"
      <div class="paragraph">
        <p>docdir= docfile=attr-masking.adoc user-home=.</p>
      </div>
    "#}
  );
}

#[test]
fn test_image_url_resolution_no_data_uri() {
  let stdout = run_input(
    &["--embedded", "--safe-mode", "safe"],
    adoc! {r#"
      :imagesdir: ./local/images

      // 1. Block image absolute URL ignores imagesdir
      image::http://example.org/images/tiger.png[Tiger]

      // 2. Inline image absolute URL ignores imagesdir
      Look at this image:http://example.org/inline/cat.png[Cat] picture.

      // 3. imagesdir as URL, relative target resolves against it
      :imagesdir: http://cdn.example.org/assets

      image::logo.png[Logo]

      // 4. Video absolute URL ignores imagesdir
      :imagesdir: assets

      // TODO: not implemented yet
      // video::http://example.org/videos/demo.mp4[]

      // 5. Audio absolute URL ignores imagesdir
      // TODO: not implemented yet
      // audio::http://example.org/audio/podcast.mp3[]
    "#},
  );
  expect_eq!(
    stdout.trim(),
    html! {r#"
      <div class="imageblock">
        <div class="content">
          <img src="http://example.org/images/tiger.png" alt="Tiger">
        </div>
      </div>
      <div class="paragraph">
        <p>Look at this <span class="image"><img src="http://example.org/inline/cat.png" alt="Cat"></span> picture.</p>
      </div>
      <div class="imageblock">
        <div class="content">
          <img src="http://cdn.example.org/assets/logo.png" alt="Logo">
        </div>
      </div>
    "#} // <div class="videoblock">
        //   <div class="content">
        //     <video src="http://example.org/videos/demo.mp4" controls>Your browser does not support the video tag.</video>
        //   </div>
        // </div>
        // <div class="audioblock">
        //   <div class="content">
        //     <audio src="http://example.org/audio/podcast.mp3" controls>Your browser does not support the audio tag.</audio>
        //   </div>
        // </div>
  );
}

// run on linux (CI) only for speed in local dev
#[cfg(target_os = "linux")]
#[test]
fn test_remote_imagesdir_data_uri() {
  let stdout = run_input(
    &[
      "--embedded",
      "--safe-mode",
      "safe",
      "--attribute",
      "allow-uri-read",
    ],
    adoc! {r#"
      :imagesdir: https://gist.githubusercontent.com/jaredh159/be5b7f2292b044681264cadc68bb0b42/raw/1ea10ecfbe1153a9c37d04fe69b3f112a04558cf
      :data-uri:

      // Relative target resolved against URL imagesdir, fetched, embedded as base64
      image::mini.svg[]
    "#},
  );
  expect_eq!(
    stdout.trim(),
    html! {r#"
      <div class="imageblock">
        <div class="content">
          <img src="data:image/svg+xml;base64,PHN2ZyB4bWxucz0iaHR0cDovL3d3dy53My5vcmcvMjAwMC9zdmciPjxwYXRoIGQ9Ik0wIDAiLz48L3N2Zz4=" alt="mini">
        </div>
      </div>
    "#}
  );
}

#[test]
fn test_iconsdir_url_no_data_uri() {
  let stdout = run_input(
    &["--embedded", "--safe-mode", "safe"],
    adoc! {r#"
      :icons:
      :iconsdir: http://cdn.example.org/icons
      :icontype: png

      // Icon resolves to URL/name.icontype
      NOTE: This note has a remote icon URL.

      // Custom icon also resolves against URL iconsdir
      [TIP,icon=custom-tip]
      Custom icon resolves to URL too.
    "#},
  );
  expect_eq!(
    stdout.trim(),
    html! {r#"
      <div class="admonitionblock note">
        <table>
          <tr>
            <td class="icon"><img src="http://cdn.example.org/icons/note.png" alt="Note"></td>
            <td class="content">This note has a remote icon URL.</td>
          </tr>
        </table>
      </div>
      <div class="admonitionblock tip">
        <table>
          <tr>
            <td class="icon"><img src="http://cdn.example.org/icons/custom-tip.png" alt="Tip"></td>
            <td class="content">Custom icon resolves to URL too.</td>
          </tr>
        </table>
      </div>
    "#}
  );
}

#[test]
fn test_iconsdir_url_data_uri_no_allow_uri_read() {
  let stdout = run_input(
    &["--embedded", "--safe-mode", "safe"],
    adoc! {r#"
      :icons:
      :iconsdir: http://cdn.example.org/icons
      :icontype: gif
      :data-uri:

      // data-uri set but no allow-uri-read: returns raw URL, no embedding
      WARNING: Cannot embed without allow-uri-read.
    "#},
  );
  expect_eq!(
    stdout.trim(),
    html! {r#"
      <div class="admonitionblock warning">
        <table>
          <tr>
            <td class="icon"><img src="http://cdn.example.org/icons/warning.gif" alt="Warning"></td>
            <td class="content">Cannot embed without allow-uri-read.</td>
          </tr>
        </table>
      </div>
    "#}
  );
}

// run on linux (CI) only for speed in local dev
#[cfg(target_os = "linux")]
#[test]
fn test_iconsdir_url_data_uri_allow_uri_read() {
  let stdout = run_input(
    &[
      "--embedded",
      "--safe-mode",
      "safe",
      "--attribute",
      "allow-uri-read",
    ],
    adoc! {r#"
      :icons:
      :iconsdir: https://raw.githubusercontent.com/jaredh159/asciidork/refs/heads/master/cli/tests/all/fixtures/images/icons
      :icontype: gif
      :data-uri:

      // Remote icon fetched and embedded as base64
      NOTE: This icon should be embedded.
    "#},
  );
  expect_eq!(
    stdout.trim(),
    html! {r#"
      <div class="admonitionblock note">
        <table>
          <tr>
            <td class="icon">
              <img src="data:image/gif;base64,R0lGODdhAQABAIABAAAAACEHbSwAAAAAAQABAAACAkwBADs=" alt="Note">
            </td>
            <td class="content">This icon should be embedded.</td>
          </tr>
        </table>
      </div>
    "#}
  );
}

#[test]
fn test_iconsdir_url_secure_mode_blocks_data_uri_embed() {
  let stdout = run_input(
    &[
      "--embedded",
      "--safe-mode",
      "secure",
      "--attribute",
      "icons",
      "--attribute",
      "allow-uri-read",
    ],
    adoc! {r#"
      :iconsdir: http://cdn.example.org/icons
      :icontype: png
      :data-uri:

      // SECURE mode: data-uri embedding blocked, returns URL path instead
      CAUTION: In secure mode, no embedding occurs.
    "#},
  );
  expect_eq!(
    stdout.trim(),
    html! {r#"
      <div class="admonitionblock caution">
        <table>
          <tr>
            <td class="icon"><img src="http://cdn.example.org/icons/caution.png" alt="Caution"></td>
            <td class="content">In secure mode, no embedding occurs.</td>
          </tr>
        </table>
      </div>
    "#}
  );
}
