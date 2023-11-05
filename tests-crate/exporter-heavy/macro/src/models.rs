use traitenum;

mod name_enum;
mod alpha;

traitenum::gen::models![
    name_enum::MODEL,
    alpha::name_enum::MODEL,
    alpha::bravo::column_enum::MODEL,
];