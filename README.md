# bevy_av1

Bevy decoder for [AV1](https://aomedia.org/av1-features/) video in an
[IVF](https://wiki.multimedia.cx/index.php/Duck_IVF) or MP4 container.

Transcode videos into either format using [ffmpeg](https://trac.ffmpeg.org/wiki/Encode/AV1), e.g.:
```sh
ffmpeg -i <input.mp4> -pix_fmt yuv420p -c:v librav1e -an -quality quality <output.ivf>

ffmpeg -i <input.mp4> -pix_fmt yuv420p -c:v librav1e -an -quality quality <output.mp4>
```

## Examples

```sh
cargo run --example demo2d  # 2d IVF over HTTPS
cargo run --example demo3d  # 3d local IVF
cargo run --example pip     # local IVF and MP4 simultaneously
cargo run --example custom  # custom video source (NTSC static simulator)
```
