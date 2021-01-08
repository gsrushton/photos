* Make DB inserts + last_row_id lookup concurrency safe
* Add person date of birth
  * Needs a front-end
* Have appearance image generation take a size parameter
  * Needs exposing to the web API
* Move remaining db queries under db/model

```sh
PHOTOSD_LOG=debug \
    build/target/release/photosd \
        --db-file-path /tmp/photos.db \
        --photo-file-path /tmp/photos \
        --thumb-file-path /tmp/thumbs \
        --static-dir-path build/docker/photosd/share/www \
        --face-landmark-predictor-model-file-path build/docker/photosd/share/shape_predictor_68_face_landmarks.dat \
        --face-encoder-model-file-path build/docker/photosd/share/dlib_face_recognition_resnet_model_v1.dat
```
