use anyhow::Result;
use firedbg_lib::fire;
use rusqlite::{Connection, Row};
use sea_query::{
    ColumnDef, Expr, ForeignKey, Iden, InsertStatement, Query, SchemaStatementBuilder,
    SelectStatement, SqliteQueryBuilder, Table, TableCreateStatement,
};
use sea_query_rusqlite::{RusqliteBinder, RusqliteValues};

fn main() -> Result<()> {
    let conn = &Connection::open_in_memory()?;

    init_schema(conn)?;

    insert_sample_data(conn)?;

    perform_select(conn)?;

    Ok(())
}

fn init_schema(conn: &Connection) -> Result<()> {
    let customer_table = create_customer_table();
    let film_table = create_film_table();
    let rental_table = create_rental_table();

    let customer_table_res = conn.execute(&build_schema_stmt(customer_table), [])?;
    fire::dbg!(customer_table_res);

    let film_table_res = conn.execute(&build_schema_stmt(film_table), [])?;
    fire::dbg!(film_table_res);

    let rental_table_res = conn.execute(&build_schema_stmt(rental_table), [])?;
    fire::dbg!(rental_table_res);

    Ok(())
}

fn insert_sample_data(conn: &Connection) -> Result<()> {
    let (sql, values) = build_query_stmt(insert_customers());
    let customer_insert_res = conn.execute(sql.as_str(), &*values.as_params())?;
    fire::dbg!(customer_insert_res);

    let (sql, values) = build_query_stmt(insert_films());
    let film_insert_res = conn.execute(sql.as_str(), &*values.as_params())?;
    fire::dbg!(film_insert_res);

    let (sql, values) = build_query_stmt(insert_rentals());
    let rental_insert_res = conn.execute(sql.as_str(), &*values.as_params())?;
    fire::dbg!(rental_insert_res);

    Ok(())
}

fn perform_select(conn: &Connection) -> Result<()> {
    let (sql, values) = build_query_stmt(select_overdue_rentals());
    let mut stmt = conn.prepare(sql.as_str())?;
    let mut rows = stmt.query(&*values.as_params())?;

    while let Some(row) = rows.next()? {
        println!("{:?}", OverdueRental::from(row));
    }

    Ok(())
}

fn create_customer_table() -> TableCreateStatement {
    Table::create()
        .table(Customer::Table)
        .if_not_exists()
        .col(
            ColumnDef::new(Customer::Id)
                .integer()
                .not_null()
                .auto_increment()
                .primary_key(),
        )
        .col(ColumnDef::new(Customer::FirstName).string().not_null())
        .col(ColumnDef::new(Customer::LastName).string().not_null())
        .col(ColumnDef::new(Customer::Email).string().not_null())
        .to_owned()
}

fn create_film_table() -> TableCreateStatement {
    Table::create()
        .table(Film::Table)
        .if_not_exists()
        .col(
            ColumnDef::new(Film::Id)
                .integer()
                .not_null()
                .auto_increment()
                .primary_key(),
        )
        .col(ColumnDef::new(Film::Title).string().not_null())
        .col(ColumnDef::new(Film::Description).string().not_null())
        .col(ColumnDef::new(Film::ReleaseYear).string().not_null())
        .col(ColumnDef::new(Film::RentalRate).double().not_null())
        .to_owned()
}

fn create_rental_table() -> TableCreateStatement {
    Table::create()
        .table(Rental::Table)
        .if_not_exists()
        .col(
            ColumnDef::new(Rental::Id)
                .integer()
                .not_null()
                .auto_increment()
                .primary_key(),
        )
        .col(ColumnDef::new(Rental::RentalDate).date().not_null())
        .col(ColumnDef::new(Rental::CustomerId).integer().not_null())
        .col(ColumnDef::new(Rental::FilmId).integer().not_null())
        .col(ColumnDef::new(Rental::DueDate).date().not_null())
        .col(ColumnDef::new(Rental::ReturnDate).date())
        .foreign_key(
            ForeignKey::create()
                .name("fk_rental_customer")
                .from(Rental::Table, Rental::CustomerId)
                .to(Customer::Table, Customer::Id),
        )
        .foreign_key(
            ForeignKey::create()
                .name("fk_rental_film")
                .from(Rental::Table, Rental::FilmId)
                .to(Film::Table, Film::Id),
        )
        .to_owned()
}

fn build_schema_stmt<T>(stmt: T) -> String
where
    T: SchemaStatementBuilder,
{
    fire::dbg!("return", stmt.build(SqliteQueryBuilder))
}

fn insert_customers() -> InsertStatement {
    Query::insert()
        .into_table(Customer::Table)
        .columns([Customer::FirstName, Customer::LastName, Customer::Email])
        .values_panic(["Cory".into(), "Hernandez".into(), "ab@zunu.hk".into()])
        .values_panic(["Aaron".into(), "Fleming".into(), "fabas@nukem.my".into()])
        .values_panic(["Clifford".into(), "Santiago".into(), "vobibe@ho.gq".into()])
        .to_owned()
}

fn insert_films() -> InsertStatement {
    Query::insert()
        .into_table(Film::Table)
        .columns([
            Film::Title,
            Film::Description,
            Film::ReleaseYear,
            Film::RentalRate,
        ])
        .values_panic([
            "Philosopher's Stone".into(),
            "The philosopher's stone, a magical object that can turn metal into gold and produce an immortality elixir.".into(),
            "2001".into(),
            10.05.into(),
        ])
        .values_panic([
            "Chamber of Secrets".into(),
            "The Chamber of Secrets has been opened, enemies of the heir... beware.".into(),
            "2002".into(),
            8.99.into(),
        ]).to_owned()
}

fn insert_rentals() -> InsertStatement {
    Query::insert()
        .into_table(Rental::Table)
        .columns([
            Rental::RentalDate,
            Rental::CustomerId,
            Rental::FilmId,
            Rental::DueDate,
            Rental::ReturnDate,
        ])
        .values_panic([
            "2023-10-10".into(),
            1.into(),
            1.into(),
            "2023-10-17".into(),
            "2023-10-15".into(),
        ])
        .values_panic([
            "2023-10-11".into(),
            1.into(),
            2.into(),
            "2023-10-18".into(),
            Option::<String>::None.into(),
        ])
        .values_panic([
            "2023-10-12".into(),
            2.into(),
            1.into(),
            "2023-10-20".into(),
            Option::<String>::None.into(),
        ])
        .values_panic([
            "2023-10-13".into(),
            2.into(),
            2.into(),
            "2023-10-21".into(),
            Option::<String>::None.into(),
        ])
        .to_owned()
}

fn select_overdue_rentals() -> SelectStatement {
    Query::select()
        .columns([Customer::FirstName, Customer::LastName])
        .columns([Film::Title])
        .columns([Rental::RentalDate, Rental::DueDate, Rental::ReturnDate])
        .from(Rental::Table)
        .inner_join(
            Customer::Table,
            Expr::col((Customer::Table, Customer::Id)).equals((Rental::Table, Rental::CustomerId)),
        )
        .inner_join(
            Film::Table,
            Expr::col((Film::Table, Film::Id)).equals((Rental::Table, Rental::FilmId)),
        )
        .and_where(Expr::col((Rental::Table, Rental::ReturnDate)).is_null())
        .and_where(Expr::col((Rental::Table, Rental::DueDate)).gt("2023-10-18"))
        .to_owned()
}

fn build_query_stmt<T>(stmt: T) -> (String, RusqliteValues)
where
    T: RusqliteBinder,
{
    fire::dbg!("return", stmt.build_rusqlite(SqliteQueryBuilder))
}

#[derive(Iden)]
enum Customer {
    Table,
    Id,
    FirstName,
    LastName,
    Email,
}

#[derive(Iden)]
enum Film {
    Table,
    Id,
    Title,
    Description,
    ReleaseYear,
    RentalRate,
}

#[derive(Iden)]
enum Rental {
    Table,
    Id,
    RentalDate,
    CustomerId,
    FilmId,
    DueDate,
    ReturnDate,
}

#[derive(Debug)]
struct OverdueRental {
    first_name: String,
    last_name: String,
    title: String,
    rental_date: String,
    due_date: String,
    return_date: Option<String>,
}

impl From<&Row<'_>> for OverdueRental {
    fn from(row: &Row) -> Self {
        fire::dbg!(
            "return",
            Self {
                first_name: row.get_unwrap("first_name"),
                last_name: row.get_unwrap("last_name"),
                title: row.get_unwrap("title"),
                rental_date: row.get_unwrap("rental_date"),
                due_date: row.get_unwrap("due_date"),
                return_date: row.get_unwrap("return_date"),
            }
        )
    }
}
