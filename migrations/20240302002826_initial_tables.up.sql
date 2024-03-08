-- Add up migration script here
CREATE TYPE loan_status AS ENUM ('pending', 'rejected', 'approved', 'repaid', 'defaulted', 'cancelled');
CREATE TABLE "user" (
	id uuid NOT NULL PRIMARY KEY default gen_random_uuid(),
	username TEXT NOT NULL UNIQUE,
	email TEXT NOT NULL UNIQUE,
	phone TEXT UNIQUE,
	password_hash TEXT NOT NULL,
	created_at timestamptz NOT NULL DEFAULT NOW(),
	updated_at timestamptz NOT NULL DEFAULT NOW()
);

create table lender (
	id uuid NOT NULL PRIMARY KEY,
	user_id uuid not null,
	foreign key (user_id) references "user"(id),
	created_at timestamptz NOT NULL DEFAULT NOW(),
	updated_at timestamptz NOT NULL DEFAULT NOW()
);

create table borrower (
	id uuid NOT NULL PRIMARY KEY,
	user_id uuid not null,
	foreign key (user_id) references "user"(id),
	created_at timestamptz NOT NULL DEFAULT NOW(),
	updated_at timestamptz NOT NULL DEFAULT NOW()
);

create table offer (
	id uuid NOT NULL PRIMARY KEY,
	lender_id uuid not null,
	interest_rate double precision not null default 0,
	min_amount double precision not null default 0,
	max_amount double precision not null default 0,
	min_duration int not null default 0,
	max_duration int not null default 0,
	created_at timestamptz NOT NULL DEFAULT NOW(),
	updated_at timestamptz NOT NULL DEFAULT NOW(),

	foreign key (lender_id) references lender(id)
);


create table loan_request (
	id uuid NOT NULL PRIMARY KEY,
	borrower_id uuid not null,
	foreign key (borrower_id) references borrower(id),
	lender_id uuid not null,
	foreign key (lender_id) references lender(id),
	amount_requested double precision not null default 0,
	amount_lent double precision not null default 0,
	interest_rate double precision not null default 0,
	accrued_interest double precision not null default 0,
	total_paid double precision not null default 0,
	outstanding_amount double precision not null default 0,
	loan_term int not null default 0,
	status loan_status not null,
	created_at timestamptz NOT NULL DEFAULT NOW(),
	updated_at timestamptz NOT NULL DEFAULT NOW()
);

create table payment (
	 id uuid NOT NULL PRIMARY KEY,
	 amount double precision not null default 0,
	 payment_reference TEXT not null,
	 payment_provider TEXT not null,
	 paid_by uuid not null,
	 payment_date timestamptz not null,
	 created_at timestamptz NOT NULL DEFAULT NOW(),
	 updated_at timestamptz NOT NULL DEFAULT NOW()
);

create table loan_transaction (
	id uuid NOT NULL PRIMARY KEY,
	lender_id uuid not null,
	payment_id uuid not null,
	loan_request_id uuid not null,
	amount_paid double precision not null default 0,
	interest_paid double precision not null default 0,
	repayment_date timestamptz not null,
	created_at timestamptz NOT NULL DEFAULT NOW(),
	updated_at timestamptz NOT NULL DEFAULT NOW(),

	foreign key (payment_id) references payment(id),
	foreign key (lender_id) references lender(id),
	foreign key (loan_request_id) references loan_request(id)
);

create table contract (
	id uuid NOT NULL PRIMARY KEY,
	lender_id uuid not null,
	borrower_id uuid not null,
	loan_request_id uuid not null,
	contract_terms TEXT not null,
	start_date timestamptz not null,
	end_date timestamptz not null,
	created_at timestamptz NOT NULL DEFAULT NOW(),
	updated_at timestamptz NOT NULL DEFAULT NOW(),

	foreign key (lender_id) references lender(id),
	foreign key (borrower_id) references borrower(id),
	foreign key (loan_request_id) references loan_request(id)
);

create table collateral (
	id uuid NOT NULL PRIMARY KEY,
	loan_request_id uuid not null,
	redeem_script TEXT not null,
	multisig_address TEXT not null,
	bitcoin_amount double precision not null default 0,
	value_in_usd double precision not null default 0,
	created_at timestamptz NOT NULL DEFAULT NOW(),
	updated_at timestamptz NOT NULL DEFAULT NOW(),

	foreign key (loan_request_id) references loan_request(id)
);


