use test_utils::*;

assert_html!(
  basic_audio_macro,
  adoc! {r#"
    audio::podcast.mp3[]
  "#},
  html! {r#"
    <div class="audioblock">
      <div class="content">
        <audio src="podcast.mp3" controls>
          Your browser does not support the audio tag.
        </audio>
      </div>
    </div>
  "#}
);

assert_html!(
  audio_macro_with_title,
  adoc! {r#"
    .Ocean waves
    audio::ocean.wav[]
  "#},
  html! {r#"
    <div class="audioblock">
      <div class="title">Ocean waves</div>
      <div class="content">
        <audio src="ocean.wav" controls>
          Your browser does not support the audio tag.
        </audio>
      </div>
    </div>
  "#}
);

assert_html!(
  audio_macro_with_id_and_role,
  adoc! {r#"
    [#my-audio.featured]
    audio::track.mp3[]
  "#},
  html! {r#"
    <div id="my-audio" class="audioblock featured">
      <div class="content">
        <audio src="track.mp3" controls>
          Your browser does not support the audio tag.
        </audio>
      </div>
    </div>
  "#}
);

assert_html!(
  audio_macro_imagesdir,
  adoc! {r#"
    :imagesdir: assets

    audio::podcast.mp3[]
  "#},
  html! {r#"
    <div class="audioblock">
      <div class="content">
        <audio src="assets/podcast.mp3" controls>
          Your browser does not support the audio tag.
        </audio>
      </div>
    </div>
  "#}
);

assert_html!(
  audio_macro_url_target_bypasses_imagesdir,
  adoc! {r#"
    :imagesdir: assets

    audio::http://example.org/podcast.mp3[]
  "#},
  html! {r#"
    <div class="audioblock">
      <div class="content">
        <audio src="http://example.org/podcast.mp3" controls>
          Your browser does not support the audio tag.
        </audio>
      </div>
    </div>
  "#}
);

assert_html!(
  audio_macro_all_options,
  adoc! {r#"
    audio::podcast.mp3[options="autoplay,nocontrols,loop"]
  "#},
  html! {r#"
    <div class="audioblock">
      <div class="content">
        <audio src="podcast.mp3" autoplay loop>
          Your browser does not support the audio tag.
        </audio>
      </div>
    </div>
  "#}
);

assert_html!(
  audio_macro_autoplay,
  adoc! {r#"
    audio::track.mp3[opts=autoplay]
  "#},
  html! {r#"
    <div class="audioblock">
      <div class="content">
        <audio src="track.mp3" autoplay controls>
          Your browser does not support the audio tag.
        </audio>
      </div>
    </div>
  "#}
);

assert_html!(
  audio_macro_loop,
  adoc! {r#"
    audio::track.mp3[opts=loop]
  "#},
  html! {r#"
    <div class="audioblock">
      <div class="content">
        <audio src="track.mp3" loop controls>
          Your browser does not support the audio tag.
        </audio>
      </div>
    </div>
  "#}
);

assert_html!(
  audio_macro_start_time,
  adoc! {r#"
    audio::podcast.mp3[start=30]
  "#},
  html! {r#"
    <div class="audioblock">
      <div class="content">
        <audio src="podcast.mp3#t=30" controls>
          Your browser does not support the audio tag.
        </audio>
      </div>
    </div>
  "#}
);

assert_html!(
  audio_macro_end_time,
  adoc! {r#"
    audio::podcast.mp3[end=60]
  "#},
  html! {r#"
    <div class="audioblock">
      <div class="content">
        <audio src="podcast.mp3#t=,60" controls>
          Your browser does not support the audio tag.
        </audio>
      </div>
    </div>
  "#}
);

assert_html!(
  audio_macro_start_and_end_time,
  adoc! {r#"
    audio::podcast.mp3[start=30,end=90]
  "#},
  html! {r#"
    <div class="audioblock">
      <div class="content">
        <audio src="podcast.mp3#t=30,90" controls>
          Your browser does not support the audio tag.
        </audio>
      </div>
    </div>
  "#}
);

assert_html!(
  audio_macro_combined_attributes,
  adoc! {r#"
    [#intro.highlight]
    .Introduction segment
    audio::interview.mp3[start=10,end=120,opts="autoplay,loop"]
  "#},
  html! {r#"
    <div id="intro" class="audioblock highlight">
      <div class="title">Introduction segment</div>
      <div class="content">
        <audio src="interview.mp3#t=10,120" autoplay loop controls>
          Your browser does not support the audio tag.
        </audio>
      </div>
    </div>
  "#}
);
