# bevy_av1

Bevy decoder for [AV1](https://aomedia.org/av1-features/) video in an
[IVF](https://wiki.multimedia.cx/index.php/Duck_IVF) container.

Transcode videos into this format using ffmpeg:
```sh
ffmpeg -i <input.mp4> -pix_fmt yuv420p -c:v librav1e -an -quality quality <output.ivf>
```
