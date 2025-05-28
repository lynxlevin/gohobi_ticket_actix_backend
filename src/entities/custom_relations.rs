use sea_orm::{Related, RelationDef, RelationTrait};

use crate::{
    diaries_diarytagrelation,
    prelude::{DiariesDiary, DiariesDiarytag},
};

impl Related<DiariesDiarytag> for DiariesDiary {
    fn to() -> RelationDef {
        diaries_diarytagrelation::Relation::DiariesDiarytag.def()
    }
    fn via() -> Option<RelationDef> {
        Some(diaries_diarytagrelation::Relation::DiariesDiary.def().rev())
    }
}
