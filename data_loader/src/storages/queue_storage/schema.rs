table! {
    delegations (stake_acc) {
        stake_acc -> Text,
        vote_acc -> Nullable<Text>,
    }
}

table! {
    downloading_statuses (id) {
        id -> Int4,
        key -> Nullable<Varchar>,
        downloading_status -> Nullable<Varchar>,
    }
}

table! {
    epochs (epoch) {
        epoch -> Int4,
        first_slot -> Nullable<Int4>,
        last_slot -> Nullable<Int4>,
        first_block -> Nullable<Int4>,
        last_block -> Nullable<Int4>,
        first_block_raw -> Nullable<Text>,
        last_block_raw -> Nullable<Text>,
        first_block_json -> Nullable<Jsonb>,
        last_block_json -> Nullable<Jsonb>,
        stakes -> Nullable<Jsonb>,
        rewards_parsing_status -> Nullable<Int4>,
    }
}

table! {
    signatures (program, signature) {
        signature -> Varchar,
        slot -> Nullable<Int4>,
        err -> Nullable<Text>,
        memo -> Nullable<Text>,
        block_time -> Nullable<Int4>,
        confirmation_status -> Nullable<Varchar>,
        loading_status -> Nullable<Int4>,
        program -> Varchar,
        potential_gap_start -> Nullable<Bool>,
    }
}

table! {
    transactions (signature) {
        slot -> Nullable<Int4>,
        transaction -> Nullable<Text>,
        block_time -> Nullable<Int4>,
        parsing_status -> Nullable<Int4>,
        signature -> Varchar,
    }
}

allow_tables_to_appear_in_same_query!(
    delegations,
    downloading_statuses,
    epochs,
    signatures,
    transactions,
);
