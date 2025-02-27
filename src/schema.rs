// @generated automatically by Diesel CLI.

diesel::table! {
    blocks (hash) {
        hash -> Text,
        timestamp -> Timestamp,
        chain_from -> Int8,
        chain_to -> Int8,
        height -> Int8,
        nonce -> Text,
        version -> Text,
        dep_state_hash -> Text,
        txs_hash -> Text,
        tx_number -> Int8,
        target -> Text,
        ghost_uncles -> Jsonb,
        main_chain -> Bool,
        deps -> Array<Nullable<Text>>,
    }
}

diesel::table! {
    events (id) {
        id -> Int4,
        tx_id -> Text,
        contract_address -> Text,
        event_index -> Int4,
        fields -> Jsonb,
    }
}

diesel::table! {
    loan_actions (id) {
        id -> Int4,
        loan_subcontract_id -> Varchar,
        loan_id -> Nullable<Numeric>,
        by -> Varchar,
        timestamp -> Timestamp,
        action_type -> Int2,
    }
}

diesel::table! {
    loan_details (id) {
        id -> Int4,
        loan_subcontract_id -> Varchar,
        lending_token_id -> Varchar,
        collateral_token_id -> Varchar,
        lending_amount -> Numeric,
        collateral_amount -> Numeric,
        interest_rate -> Numeric,
        duration -> Numeric,
        lender -> Varchar,
    }
}

diesel::table! {
    processor_status (processor) {
        #[max_length = 50]
        processor -> Varchar,
        last_timestamp -> Int8,
    }
}

diesel::table! {
    transactions (tx_hash) {
        tx_hash -> Text,
        unsigned -> Jsonb,
        script_execution_ok -> Bool,
        contract_inputs -> Jsonb,
        generated_outputs -> Jsonb,
        input_signatures -> Array<Nullable<Text>>,
        script_signatures -> Array<Nullable<Text>>,
        created_at -> Nullable<Timestamptz>,
        updated_at -> Nullable<Timestamptz>,
        main_chain -> Bool,
        block_hash -> Text,
    }
}

diesel::joinable!(transactions -> blocks (block_hash));

diesel::allow_tables_to_appear_in_same_query!(
    blocks,
    events,
    loan_actions,
    loan_details,
    processor_status,
    transactions,
);
