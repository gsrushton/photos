
CREATE TABLE people (
  id           INTEGER PRIMARY KEY NOT NULL,
  first_name   TEXT NOT NULL,
  middle_names TEXT,
  surname      TEXT NOT NULL,
  display_name TEXT,
  dob          DATE
);

CREATE TABLE photos (
  id                INTEGER PRIMARY KEY NOT NULL,
  digest            BLOB NOT NULL,
  file_name         TEXT NOT NULL,
  image_width       INTEGER NOT NULL,
  image_height      INTEGER NOT NULL,
  thumb_width       INTEGER NOT NULL,
  thumb_height      INTEGER NOT NULL,
  original_datetime DATETIME,
  upload_datetime   DATETIME NOT NULL
);

CREATE UNIQUE INDEX photos_by_digest ON photos(digest);

CREATE TABLE appearances (
  id            INTEGER PRIMARY KEY NOT NULL,
  person        INTEGER NOT NULL REFERENCES people(id),
  photo         INTEGER NOT NULL REFERENCES photos(id),
  reference     BOOLEAN NOT NULL,
  top           INTEGER NOT NULL,
  left          INTEGER NOT NULL,
  bottom        INTEGER NOT NULL,
  right         INTEGER NOT NULL,
  face_encoding BLOB NOT NULL
);

CREATE TABLE avatars (
  id         INTEGER PRIMARY KEY NOT NULL,
  person     INTEGER NOT NULL UNIQUE REFERENCES people(id),
  appearance INTEGER NOT NULL REFERENCES appearances(id)
);
