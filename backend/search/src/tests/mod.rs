use uuid::uuid;

use crate::{
    reciprocal_rank_fusion, OpenSearchHit, VectorSearchResult,
};

#[test]
fn test_rrf_merges_results_correctly() {
    let bm25 = vec![
        OpenSearchHit {
            usage_instance_id: uuid!("00000000-0000-0000-0000-000000000001"),
            bm25_score: 1.0,
        },
        OpenSearchHit {
            usage_instance_id: uuid!("00000000-0000-0000-0000-000000000002"),
            bm25_score: 0.5,
        },
    ];
    let vector = vec![
        VectorSearchResult {
            id: uuid!("00000000-0000-0000-0000-000000000001"),
            distance: 0.1,
        },
        VectorSearchResult {
            id: uuid!("00000000-0000-0000-0000-000000000003"),
            distance: 0.2,
        },
    ];

    let results = reciprocal_rank_fusion(&bm25, &vector, 60.0);

    assert_eq!(
        results[0].usage_instance_id,
        uuid!("00000000-0000-0000-0000-000000000001")
    );
    assert_eq!(results.len(), 3);
}

#[test]
fn rrf_returns_each_unique_result_once() {
    let id = uuid!("00000000-0000-0000-0000-000000000001");
    let bm25 = vec![OpenSearchHit {
        usage_instance_id: id,
        bm25_score: 7.5,
    }];
    let vector = vec![VectorSearchResult { id, distance: 0.05 }];

    let results = reciprocal_rank_fusion(&bm25, &vector, 60.0);

    assert_eq!(results.len(), 1);
    assert!(results[0].fused_score > 1.0 / 61.0);
}
