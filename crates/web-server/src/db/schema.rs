table! {
    appearances (id) {
        id -> Integer,
        person -> Integer,
        photo -> Integer,
        reference -> Bool,
        top -> Integer,
        left -> Integer,
        bottom -> Integer,
        right -> Integer,
        face_encoding -> Binary,
    }
}

table! {
    avatars (id) {
        id -> Integer,
        person -> Integer,
        appearance -> Integer,
    }
}

table! {
    people (id) {
        id -> Integer,
        first_name -> Text,
        middle_names -> Nullable<Text>,
        surname -> Text,
        display_name -> Nullable<Text>,
        dob -> Nullable<Date>,
    }
}

table! {
    photos (id) {
        id -> Integer,
        digest -> Binary,
        file_name -> Text,
        image_width -> Integer,
        image_height -> Integer,
        thumb_width -> Integer,
        thumb_height -> Integer,
        original_datetime -> Nullable<Timestamp>,
        upload_datetime -> Timestamp,
    }
}

joinable!(appearances -> people (person));
joinable!(appearances -> photos (photo));
joinable!(avatars -> appearances (appearance));
joinable!(avatars -> people (person));

allow_tables_to_appear_in_same_query!(appearances, avatars, people, photos,);
