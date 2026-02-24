use test_utils::*;

assert_html!(
  basic_video_macro,
  adoc! {r#"
    video::cats-vs-dogs.avi[]
  "#},
  html! {r#"
    <div class="videoblock">
      <div class="content">
        <video src="cats-vs-dogs.avi" controls>
          Your browser does not support the video tag.
        </video>
      </div>
    </div>
  "#}
);

assert_html!(
  video_macro_positional_attrs,
  adoc! {r#"
    video::cats-vs-dogs.avi[cats-and-dogs.png, 200, 300]
  "#},
  html! {r#"
    <div class="videoblock">
      <div class="content">
        <video src="cats-vs-dogs.avi" width="200" height="300" poster="cats-and-dogs.png" controls>
          Your browser does not support the video tag.
        </video>
      </div>
    </div>
  "#}
);

assert_html!(
  video_macro_with_id_and_role_and_title,
  adoc! {r#"
    .Product demo
    [#my-video.featured]
    video::promo.mp4[]
  "#},
  html! {r#"
    <div id="my-video" class="videoblock featured">
      <div class="title">Product demo</div>
      <div class="content">
        <video src="promo.mp4" controls>
          Your browser does not support the video tag.
        </video>
      </div>
    </div>
  "#}
);

assert_html!(
  video_macro_imagesdir,
  adoc! {r#"
    :imagesdir: assets

    video::cats-vs-dogs.avi[cats-and-dogs.png, 200, 300]

    video::http://example.org/videos/cats-vs-dogs.avi[]
  "#},
  contains:
    r##"src="assets/cats-vs-dogs.avi""##,
    r##"poster="assets/cats-and-dogs.png""##,
    r##"src="http://example.org/videos/cats-vs-dogs.avi""##,
);

assert_html!(
  video_macro_float_and_align,
  adoc! {r#"
    video::cats-vs-dogs.avi[cats-and-dogs.png,float=right]

    video::cats-vs-dogs.avi[cats-and-dogs.png,align=center]
  "#},
  contains:
    r##"class="videoblock right""##,
    r##"class="videoblock text-center""##,
);

assert_html!(
  video_macro_all_options,
  adoc! {r#"
    video::cats-vs-dogs.avi[options="autoplay,muted,nocontrols,loop",preload="metadata"]
  "#},
  html! {r#"
    <div class="videoblock">
      <div class="content">
        <video src="cats-vs-dogs.avi" autoplay muted loop preload="metadata">
          Your browser does not support the video tag.
        </video>
      </div>
    </div>
  "#}
);

assert_html!(
  video_macro_time_anchors,
  adoc! {r#"
    video::cats-vs-dogs.avi[start="30"]

    video::cats-vs-dogs.avi[end="30"]

    video::cats-vs-dogs.avi[start="30",end="60"]
  "#},
  contains:
    r##"src="cats-vs-dogs.avi#t=30" controls"##,
    r##"src="cats-vs-dogs.avi#t=,30" controls"##,
    r##"src="cats-vs-dogs.avi#t=30,60" controls"##,
);

assert_html!(
  video_macro_native_combined,
  adoc! {r#"
    [#promo.highlight]
    .Product walkthrough
    video::demo.mp4[poster=thumb.png,width=800,height=600,start=10,end=300,opts="autoplay,muted,loop"]
  "#},
  html! {r#"
    <div id="promo" class="videoblock highlight">
      <div class="title">Product walkthrough</div>
      <div class="content">
        <video src="demo.mp4#t=10,300" width="800" height="600" poster="thumb.png" autoplay muted loop controls>
          Your browser does not support the video tag.
        </video>
      </div>
    </div>
  "#}
);

assert_html!(
  video_macro_youtube,
  adoc! {r#"
    video::U8GBXvdmHT4/PLg7s6cbtAD15Das5LK9mXt_g59DLWxKUe[youtube, 640, 360, start=60, options="autoplay,muted,modest", theme=light]
  "#},
  html! {r#"
    <div class="videoblock">
      <div class="content">
        <iframe width="640" height="360" src="https://www.youtube.com/embed/U8GBXvdmHT4?list=PLg7s6cbtAD15Das5LK9mXt_g59DLWxKUe&amp;autoplay=1&amp;mute=1&amp;modestbranding=1&amp;rel=0&amp;theme=light&amp;start=60" frameborder="0" allowfullscreen></iframe>
      </div>
    </div>
  "#}
);

assert_html!(
  video_macro_youtube_dynamic_playlist,
  adoc! {r#"
    video::SCZF6I-Rc4I,AsKGOeonbIs,HwrPhOp6-aM[youtube, 640, 360, start=60, options=autoplay]
  "#},
  html! {r#"
    <div class="videoblock">
      <div class="content">
        <iframe width="640" height="360" src="https://www.youtube.com/embed/SCZF6I-Rc4I?playlist=SCZF6I-Rc4I,AsKGOeonbIs,HwrPhOp6-aM&amp;autoplay=1&amp;rel=0&amp;start=60" frameborder="0" allowfullscreen></iframe>
      </div>
    </div>
  "#}
);

assert_html!(
  video_macro_youtube_nofullscreen,
  adoc! {r#"
    video::U8GBXvdmHT4[youtube, 640, 360, options=nofullscreen]
  "#},
  html! {r#"
    <div class="videoblock">
      <div class="content">
        <iframe width="640" height="360" src="https://www.youtube.com/embed/U8GBXvdmHT4?rel=0&amp;fs=0" frameborder="0"></iframe>
      </div>
    </div>
  "#}
);

assert_html!(
  video_macro_youtube_nocontrols,
  adoc! {r#"
    video::U8GBXvdmHT4[youtube, 640, 360, options=nocontrols]
  "#},
  html! {r#"
    <div class="videoblock">
      <div class="content">
        <iframe width="640" height="360" src="https://www.youtube.com/embed/U8GBXvdmHT4?rel=0&amp;controls=0" frameborder="0" allowfullscreen></iframe>
      </div>
    </div>
  "#}
);

// loop requires playlist param (uses video ID when no explicit playlist)
assert_html!(
  video_macro_youtube_loop,
  adoc! {r#"
    video::U8GBXvdmHT4[youtube, 640, 360, options=loop]
  "#},
  html! {r#"
    <div class="videoblock">
      <div class="content">
        <iframe width="640" height="360" src="https://www.youtube.com/embed/U8GBXvdmHT4?loop=1&amp;rel=0&amp;playlist=U8GBXvdmHT4" frameborder="0" allowfullscreen></iframe>
      </div>
    </div>
  "#}
);

assert_html!(
  video_macro_youtube_url_params,
  adoc! {r#"
    video::U8GBXvdmHT4[youtube, 640, 360, lang=fr]

    video::U8GBXvdmHT4[youtube, 640, 360, options=related]

    video::U8GBXvdmHT4[youtube, 640, 360, end=120]

    video::RvRhUHTV_8k[youtube, 640, 360, list=PLDitloyBcHOm49bxNhvGgg0f9NRZ5lSaP]

    video::RvRhUHTV_8k[youtube, 640, 360, list=PLDitloyBcHOm49bxNhvGgg0f9NRZ5lSaP]

    video::RvRhUHTV_8k[youtube, 640, 360, playlist="_SvwdK_HibQ,SGqg_ZzThDU"]
  "#},
  contains:
    r#"hl=fr"#,
    r##"?rel=1""##,
    r#"end=120"#,
    r#"list=PLDitloyBcHOm49bxNhvGgg0f9NRZ5lSaP"#,
    r#"playlist=RvRhUHTV_8k,_SvwdK_HibQ,SGqg_ZzThDU"#,
);

assert_html!(
  video_macro_vimeo,
  adoc! {r#"
    video::67480300[vimeo, 400, 300, start=60, options="autoplay,muted"]
  "#},
  html! {r#"
    <div class="videoblock">
      <div class="content">
        <iframe width="400" height="300" src="https://player.vimeo.com/video/67480300?autoplay=1&amp;muted=1#at=60" frameborder="0" allowfullscreen></iframe>
      </div>
    </div>
  "#}
);

assert_html!(
  video_macro_vimeo_hash,
  adoc! {r#"
    video::67480300/999999[vimeo, 400, 300, options=loop]

    video::67480300[vimeo, 400, 300, options=loop, hash=123456789]
  "#},
  contains:
    r#"https://player.vimeo.com/video/67480300?h=999999&amp;loop=1"#,
    r#"https://player.vimeo.com/video/67480300?h=123456789&amp;loop=1"#,
);

assert_html!(
  video_macro_vimeo_nofullscreen,
  adoc! {r#"
    video::67480300[vimeo, 400, 300, options=nofullscreen]
  "#},
  html! {r#"
    <div class="videoblock">
      <div class="content">
        <iframe width="400" height="300" src="https://player.vimeo.com/video/67480300" frameborder="0"></iframe>
      </div>
    </div>
  "#}
);

assert_html!(
  video_macro_wistia,
  adoc! {r#"
    video::be5gtsbaco[wistia,640,360,start=60,options="autoplay,loop,muted"]
  "#},
  html! {r#"
    <div class="videoblock">
      <div class="content">
        <iframe width="640" height="360" src="https://fast.wistia.com/embed/iframe/be5gtsbaco?autoPlay=true&amp;muted=true&amp;endVideoBehavior=loop&amp;time=60" frameborder="0" allowfullscreen class="wistia_embed" name="wistia_embed"></iframe>
      </div>
    </div>
  "#}
);

assert_html!(
  video_macro_wistia_reset,
  adoc! {r#"
    video::be5gtsbaco[wistia,640,360,start=60,options="autoplay,reset,muted"]
  "#},
  html! {r#"
    <div class="videoblock">
      <div class="content">
        <iframe width="640" height="360" src="https://fast.wistia.com/embed/iframe/be5gtsbaco?autoPlay=true&amp;muted=true&amp;endVideoBehavior=reset&amp;time=60" frameborder="0" allowfullscreen class="wistia_embed" name="wistia_embed"></iframe>
      </div>
    </div>
  "#}
);
