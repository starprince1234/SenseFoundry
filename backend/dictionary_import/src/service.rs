use kernel::{AppError, AppResult, Page, PageParams};
use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

use crate::{
    Headword, IsolationRules, NewHeadword, NewReferenceSense, ReferenceSense, SourceKind,
};

const SEED_SOURCE_URI: &str = "internal://sensefoundry/mvp-seed/v1";

pub struct DictionaryImportService;

impl DictionaryImportService {
    pub async fn create_headword(pool: &PgPool, input: NewHeadword) -> AppResult<Headword> {
        validate_headword(&input)?;
        let normalized = input.character.trim().to_string();
        let headword = sqlx::query_as::<_, Headword>(
            "INSERT INTO headwords \
             (character, normalized, pinyin, stroke_count, radical, traditional_form) \
             VALUES ($1, $2, $3, $4, $5, $6) \
             RETURNING id, character, normalized, pinyin, stroke_count, radical, \
                       traditional_form, created_at",
        )
        .bind(input.character)
        .bind(normalized)
        .bind(input.pinyin)
        .bind(input.stroke_count)
        .bind(input.radical)
        .bind(input.traditional_form)
        .fetch_one(pool)
        .await?;
        Ok(headword)
    }

    pub async fn list_headwords(pool: &PgPool, params: &PageParams) -> AppResult<Page<Headword>> {
        let total = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM headwords WHERE deleted_at IS NULL",
        )
        .fetch_one(pool)
        .await?;
        let items = sqlx::query_as::<_, Headword>(
            "SELECT id, character, normalized, pinyin, stroke_count, radical, \
                    traditional_form, created_at \
             FROM headwords WHERE deleted_at IS NULL \
             ORDER BY created_at, id LIMIT $1 OFFSET $2",
        )
        .bind(params.limit())
        .bind(params.offset())
        .fetch_all(pool)
        .await?;
        Ok(Page::new(items, params, Some(total)))
    }

    pub async fn create_reference_sense(
        pool: &PgPool,
        sense: NewReferenceSense,
    ) -> AppResult<ReferenceSense> {
        validate_sense(&sense)?;
        IsolationRules::validate_seed_sense(&sense)?;
        IsolationRules::check_authoritative_gate(sense.is_authoritative)?;
        let version_id = IsolationRules::validate_source_record(pool, &sense).await?;
        insert_sense(pool, &sense, version_id).await
    }

    pub async fn list_reference_senses(
        pool: &PgPool,
        params: &PageParams,
    ) -> AppResult<Page<ReferenceSense>> {
        let total = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM reference_senses WHERE deleted_at IS NULL",
        )
        .fetch_one(pool)
        .await?;
        let items = sqlx::query_as::<_, ReferenceSense>(&format!(
            "{} ORDER BY created_at, id LIMIT $1 OFFSET $2",
            reference_sense_select()
        ))
        .bind(params.limit())
        .bind(params.offset())
        .fetch_all(pool)
        .await?;
        Ok(Page::new(items, params, Some(total)))
    }

    pub async fn get_reference_sense(pool: &PgPool, id: Uuid) -> AppResult<ReferenceSense> {
        sqlx::query_as::<_, ReferenceSense>(&format!("{} AND id = $1", reference_sense_select()))
            .bind(id)
            .fetch_optional(pool)
            .await?
            .ok_or_else(|| AppError::NotFound(format!("reference sense {id}")))
    }
}

pub async fn import_seed_senses(pool: &PgPool) -> AppResult<()> {
    let mut transaction = pool.begin().await?;
    sqlx::query("SELECT pg_advisory_xact_lock(hashtext($1))")
        .bind(SEED_SOURCE_URI)
        .execute(&mut *transaction)
        .await?;
    let source_id = ensure_seed_source(&mut transaction).await?;

    for (character, glosses) in seed_data() {
        let headword_id = ensure_headword(&mut transaction, character).await?;
        for (index, gloss) in glosses.iter().enumerate() {
            let sense = NewReferenceSense {
                headword_id,
                source_id,
                sense_number: (index + 1) as i32,
                pos: Some("verb".into()),
                gloss: (*gloss).into(),
                example_text: None,
                source_kind: SourceKind::InternalSeed,
                is_authoritative: false,
                is_publishable: false,
                copyright_isolate: true,
            };
            IsolationRules::validate_seed_sense(&sense)?;
            sqlx::query(
                "INSERT INTO reference_senses \
                 (headword_id, source_id, sense_number, pos, gloss, example_text, source_kind, \
                  is_authoritative, is_publishable, copyright_isolate) \
                 SELECT $1, $2, $3, $4, $5, $6, $7, $8, $9, $10 \
                 WHERE NOT EXISTS (SELECT 1 FROM reference_senses \
                     WHERE source_id = $2 AND headword_id = $1 AND sense_number = $3 \
                       AND deleted_at IS NULL)",
            )
            .bind(sense.headword_id)
            .bind(sense.source_id)
            .bind(sense.sense_number)
            .bind(&sense.pos)
            .bind(&sense.gloss)
            .bind(&sense.example_text)
            .bind(sense.source_kind.as_db_str())
            .bind(sense.is_authoritative)
            .bind(sense.is_publishable)
            .bind(sense.copyright_isolate)
            .execute(&mut *transaction)
            .await?;
        }
    }
    transaction.commit().await?;
    Ok(())
}

async fn insert_sense(
    pool: &PgPool,
    sense: &NewReferenceSense,
    version_id: Option<Uuid>,
) -> AppResult<ReferenceSense> {
    let value = sqlx::query_as::<_, ReferenceSense>(
        "INSERT INTO reference_senses \
         (headword_id, source_id, dictionary_version_id, sense_number, pos, gloss, example_text, \
          source_kind, is_authoritative, is_publishable, copyright_isolate) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11) \
         RETURNING id, headword_id, source_id, dictionary_version_id, sense_number, pos, gloss, \
                   example_text, source_kind, is_authoritative, is_publishable, \
                   copyright_isolate, created_at",
    )
    .bind(sense.headword_id)
    .bind(sense.source_id)
    .bind(version_id)
    .bind(sense.sense_number)
    .bind(&sense.pos)
    .bind(&sense.gloss)
    .bind(&sense.example_text)
    .bind(sense.source_kind.as_db_str())
    .bind(sense.is_authoritative)
    .bind(sense.is_publishable)
    .bind(sense.copyright_isolate)
    .fetch_one(pool)
    .await?;
    Ok(value)
}

fn validate_headword(input: &NewHeadword) -> AppResult<()> {
    if input.character.trim().is_empty() {
        return Err(AppError::Unprocessable("Headword cannot be empty".into()));
    }
    if input.stroke_count.is_some_and(|count| count < 0) {
        return Err(AppError::Unprocessable(
            "Stroke count cannot be negative".into(),
        ));
    }
    Ok(())
}

fn validate_sense(sense: &NewReferenceSense) -> AppResult<()> {
    if sense.sense_number < 1 {
        return Err(AppError::Unprocessable(
            "Sense number must be positive".into(),
        ));
    }
    if sense.gloss.trim().is_empty() {
        return Err(AppError::Unprocessable("Gloss cannot be empty".into()));
    }
    if !sense.copyright_isolate {
        return Err(AppError::Unprocessable(
            "Reference senses must remain copyright-isolated".into(),
        ));
    }
    if sense.is_authoritative != (sense.source_kind == SourceKind::Authoritative) {
        return Err(AppError::Unprocessable(
            "Authoritative flag must match authoritative source kind".into(),
        ));
    }
    Ok(())
}

async fn ensure_seed_source(transaction: &mut Transaction<'_, Postgres>) -> AppResult<Uuid> {
    if let Some(id) = sqlx::query_scalar::<_, Uuid>(
        "SELECT id FROM sources WHERE uri = $1 AND deleted_at IS NULL LIMIT 1",
    )
    .bind(SEED_SOURCE_URI)
    .fetch_optional(&mut **transaction)
    .await?
    {
        return Ok(id);
    }
    Ok(sqlx::query_scalar::<_, Uuid>(
        "INSERT INTO sources \
         (uri, title, license, source_kind, copyright_status, is_authoritative) \
         VALUES ($1, $2, $3, 'internal_seed', 'public_domain', FALSE) RETURNING id",
    )
    .bind(SEED_SOURCE_URI)
    .bind("SenseFoundry MVP seed senses")
    .bind("Internal bootstrap data; non-publishable")
    .fetch_one(&mut **transaction)
    .await?)
}

async fn ensure_headword(
    transaction: &mut Transaction<'_, Postgres>,
    character: &str,
) -> AppResult<Uuid> {
    if let Some(id) = sqlx::query_scalar::<_, Uuid>(
        "SELECT id FROM headwords WHERE character = $1 AND deleted_at IS NULL LIMIT 1",
    )
    .bind(character)
    .fetch_optional(&mut **transaction)
    .await?
    {
        return Ok(id);
    }
    Ok(sqlx::query_scalar::<_, Uuid>(
        "INSERT INTO headwords (character, normalized, pinyin) VALUES ($1, $1, $2) RETURNING id",
    )
    .bind(character)
    .bind(Vec::<String>::new())
    .fetch_one(&mut **transaction)
    .await?)
}

fn reference_sense_select() -> &'static str {
    "SELECT id, headword_id, source_id, dictionary_version_id, sense_number, pos, gloss, \
            example_text, source_kind, is_authoritative, is_publishable, copyright_isolate, \
            created_at FROM reference_senses WHERE deleted_at IS NULL"
}

fn seed_data() -> [(&'static str, [&'static str; 5]); 5] {
    [
        ("打", ["用手、器具等接触后用力推移；击", "进行某种动作或活动", "拨通电话", "制作或建造", "获取或购买"]),
        ("开", ["使关闭的东西不再关闭", "启动机器或设备", "开始或举行", "开辟或扩展", "离开原处并行进"]),
        ("发", ["送出或交付", "产生或发生", "表达或发布", "扩展、生长", "显现某种状态"]),
        ("上", ["由低处向高处移动", "到达或进入某处", "安装或添加", "开始工作或学习", "达到规定数量或程度"]),
        ("下", ["由高处向低处移动", "离开交通工具", "作出命令或决定", "投入或使用", "结束工作或课程"]),
    ]
}
