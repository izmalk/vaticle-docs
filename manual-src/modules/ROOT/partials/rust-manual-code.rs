// tag::import[]
use typedb_driver::{
    concept::{Attribute, Concept, Transitivity, Value, ValueType},
    transaction::{
        concept::api::{EntityTypeAPI, ThingAPI, ThingTypeAPI},
        logic::api::RuleAPI,
    },
    Connection, DatabaseManager, Error, Options, Promise, Session, SessionType, TransactionType,
};
// end::import[]

fn main() -> Result<(), Error> {
    const DB_NAME: &str = "sample_db";
    const SERVER_ADDR: &str = "127.0.0.1:1729";

    println!("TypeDB Manual sample code");

    println!("Attempting to connect to a TypeDB Core server: {}", SERVER_ADDR);
    // tag::driver[]
    let driver = Connection::new_core(SERVER_ADDR)?;
    // end::driver[]
    // tag::databases[]
    let databases = DatabaseManager::new(driver);
    // end::databases[]
    // tag::list-db[]
    for db in databases.all()? {
        println!("{}", db.name());
    }
    // end::list-db[]
    // tag::delete-db[]
    if databases.contains(DB_NAME)? {
        let _ = databases.get(DB_NAME)?.delete();
    }
    // end::delete-db[]
    // tag::create-db[]
    let _ = databases.create(DB_NAME);
    // end::create-db[]
    if databases.contains(DB_NAME)? {
        println!("Database setup complete");
    }

    {
        // tag::define[]
        let db = databases.get(DB_NAME)?;
        {
            let session = Session::new(db, SessionType::Schema)?;
            {
                let transaction = session.transaction(TransactionType::Write)?;
                let define_query = "
                                    define
                                    email sub attribute, value string;
                                    name sub attribute, value string;
                                    friendship sub relation, relates friend;
                                    user sub entity,
                                        owns email @key,
                                        owns name,
                                        plays friendship:friend;
                                    admin sub user;
                                    ";
                transaction.query().define(define_query).resolve()?;
                transaction.commit().resolve()?;
            }
        }
        // end::define[]
    }

    {
        // tag::undefine[]
        let db = databases.get(DB_NAME)?;
        {
            let session = Session::new(db, SessionType::Schema)?;
            {
                let transaction = session.transaction(TransactionType::Write)?;
                let undefine_query = "undefine admin sub user;";
                transaction.query().undefine(undefine_query).resolve()?;
                transaction.commit().resolve()?;
            }
        }
        // end::undefine[]
    }

    {
        // tag::insert[]
        let db = databases.get(DB_NAME)?;
        {
            let session = Session::new(db, SessionType::Data)?;
            {
                let transaction = session.transaction(TransactionType::Write)?;
                let insert_query = "
                                    insert
                                    $user1 isa user, has name 'Alice', has email 'alice@vaticle.com';
                                    $user2 isa user, has name 'Bob', has email 'bob@vaticle.com';
                                    $friendship (friend:$user1, friend: $user2) isa friendship;
                                    ";
                let _ = transaction.query().insert(insert_query)?;
                transaction.commit().resolve()?;
            }
        }
        // end::insert[]
    }

    {
        // tag::match-insert[]
        let db = databases.get(DB_NAME)?;
        {
            let session = Session::new(db, SessionType::Data)?;
            {
                let transaction = session.transaction(TransactionType::Write)?;
                let match_insert_query = "
                                        match
                                        $u isa user, has name 'Bob';
                                        insert
                                        $new-u isa user, has name 'Charlie', has email 'charlie@vaticle.com';
                                        $f($u,$new-u) isa friendship;
                                        ";
                let response_count = transaction.query().insert(match_insert_query)?.count();
                if response_count == 1 {
                    transaction.commit().resolve()?;
                } else {
                    transaction.force_close();
                }
            }
        }
        // end::match-insert[]
    }

    {
        // tag::delete[]
        let db = databases.get(DB_NAME)?;
        {
            let session = Session::new(db, SessionType::Data)?;
            {
                let transaction = session.transaction(TransactionType::Write)?;
                let delete_query = "
                                    match
                                    $u isa user, has name 'Charlie';
                                    $f ($u) isa friendship;
                                    delete
                                    $f isa friendship;
                                    ";
                let _ = transaction.query().delete(delete_query).resolve();
                transaction.commit().resolve()?;
            }
        }
        // end::delete[]
    }

    {
        // tag::update[]
        let db = databases.get(DB_NAME)?;
        {
            let session = Session::new(db, SessionType::Data)?;
            {
                let transaction = session.transaction(TransactionType::Write)?;
                let update_query = "
                                    match
                                    $u isa user, has name 'Charlie', has email $e;
                                    delete
                                    $u has $e;
                                    insert
                                    $u has email 'charles@vaticle.com';
                                    ";
                let response_count = transaction.query().update(update_query)?.count();
                if response_count == 1 {
                    transaction.commit().resolve()?;
                } else {
                    transaction.force_close();
                }
            }
        }
        // end::update[]
    }

    {
        // tag::fetch[]
        let db = databases.get(DB_NAME)?;
        {
            let session = Session::new(db, SessionType::Data)?;
            {
                let transaction = session.transaction(TransactionType::Read)?;
                let fetch_query = "
                                    match
                                    $u isa user;
                                    fetch
                                    $u: name, email;
                                    ";
                let response = transaction.query().fetch(fetch_query)?;
                for (i, json) in response.enumerate() {
                    println!("User #{}: {}", (i + 1).to_string(), json.unwrap().to_string())
                }
            }
        }
        // end::fetch[]
    }

    {
        // tag::get[]
        let db = databases.get(DB_NAME)?;
        {
            let session = Session::new(db, SessionType::Data)?;
            {
                let transaction = session.transaction(TransactionType::Read)?;
                let get_query = "
                                match
                                $u isa user, has email $e;
                                get
                                $e;
                                ";
                let response = transaction.query().get(get_query)?;
                for (i, cm) in response.enumerate() {
                    let email_concept = cm.unwrap().get("e").unwrap().clone();
                    let email = match email_concept {
                        Concept::Attribute(Attribute { value: Value::String(value), .. }) => value,
                        _ => unreachable!(),
                    };
                    println!("Email #{}: {}", (i + 1).to_string(), email)
                }
            }
        }
        // end::get[]
    }

    {
        // tag::infer-rule[]
        let db = databases.get(DB_NAME)?;
        {
            let session = Session::new(db, SessionType::Schema)?;
            {
                let transaction = session.transaction(TransactionType::Write)?;
                let define_query = "
                                    define
                                    rule users:
                                    when {
                                        $u isa user;
                                    } then {
                                        $u has name 'User';
                                    };
                                    ";
                transaction.query().define(define_query).resolve()?;
                transaction.commit().resolve()?;
            }
        }
        // end::infer-rule[]
        // tag::infer-fetch[]
        let db = databases.get(DB_NAME)?;
        let options = Options::new().infer(true);
        {
            let session = Session::new(db, SessionType::Data)?;
            {
                let transaction = session.transaction_with_options(TransactionType::Read, options)?;
                let fetch_query = "
                                    match
                                    $u isa user;
                                    fetch
                                    $u: name, email;
                                    ";
                let response = transaction.query().fetch(fetch_query)?;
                for (i, json) in response.enumerate() {
                    println!("User #{}: {}", (i + 1).to_string(), json.unwrap().to_string())
                }
            }
        }
        // end::infer-fetch[]
    }

    {
        // tag::types-editing[]
        let db = databases.get(DB_NAME)?;
        {
            let session = Session::new(db, SessionType::Schema)?;
            {
                let transaction = session.transaction(TransactionType::Write)?;
                let tag = &transaction.concept().put_attribute_type("tag".to_owned(), ValueType::String).resolve()?;
                let entities = transaction
                    .concept()
                    .get_entity_type("entity".to_owned())
                    .resolve()?
                    .ok_or("No root entity")
                    .unwrap()
                    .get_subtypes(&transaction, Transitivity::Explicit)?;
                for entity in entities {
                    let mut e = entity?;
                    println!("{}", e.label);
                    if !(e.is_abstract()) {
                        let _ = e.set_owns(&transaction, tag.clone(), None, vec![]);
                    }
                }
                let _ = transaction.commit().resolve();
            }
        }
        // end::types-editing[]
    }

    {
        // tag::types-api[]
        let db = databases.get(DB_NAME)?;
        {
            let session = Session::new(db, SessionType::Schema)?;
            {
                let transaction = session.transaction(TransactionType::Write)?;
                let user = transaction
                    .concept()
                    .get_entity_type("user".to_owned())
                    .resolve()?
                    .ok_or("No root entity")
                    .unwrap();
                let mut admin = transaction.concept().put_entity_type("admin".to_owned()).resolve()?;
                drop(admin.set_supertype(&transaction, user).resolve());
                let entities = transaction
                    .concept()
                    .get_entity_type("entity".to_owned())
                    .resolve()?
                    .ok_or("No root entity")
                    .unwrap()
                    .get_subtypes(&transaction, Transitivity::Transitive)?;
                for subtype in entities {
                    println!("{}", subtype?.label);
                }
                let _ = transaction.commit().resolve();
            }
        }
        // end::types-api[]
    }

    {
        // tag::rules-api[]
        let db = databases.get(DB_NAME)?;
        {
            let session = Session::new(db, SessionType::Schema)?;
            {
                let transaction = session.transaction(TransactionType::Write)?;
                let r = transaction.logic().get_rule("users".to_owned()).resolve()?.ok_or("Rule not found.").unwrap();
                println!("Rule label: {}", r.label);
                println!("  Condition: {}", r.when.to_string());
                println!("  Conclusion: {}", r.then.to_string());
                let condition = typeql::parse_pattern("{$u isa user, has email $e; $e contains '@vaticle.com';}")?
                    .into_conjunction();
                let conclusion = typeql::parse_pattern("$u has name 'Employee'")?.into_statement();
                let mut new_rule =
                    transaction.logic().put_rule("Employee".to_string(), condition, conclusion).resolve()?;
                let rules = transaction.logic().get_rules()?;
                for rule in rules {
                    println!("{}", rule?.label);
                }
                let _ = new_rule.delete(&transaction).resolve();
                let _ = transaction.commit().resolve();
            }
        }
        // end::rules-api[]
    }

    {
        // tag::data-api[]
        let db = databases.get(DB_NAME)?;
        {
            let session = Session::new(db, SessionType::Data)?;
            {
                let transaction = session.transaction(TransactionType::Write)?;
                let user_type = transaction.concept().get_entity_type("user".to_owned()).resolve()?.unwrap();
                let users = user_type.get_instances(&transaction, Transitivity::Transitive)?;
                for user in users {
                    let user = user?;
                    println!("User:");
                    let attributes = user.get_has(&transaction, vec![], vec![])?;
                    for attribute in attributes {
                        let attribute = attribute?;
                        let value = match attribute.value {
                            Value::String(value) => value,
                            Value::Long(value) => value.to_string(),
                            Value::Double(value) => value.to_string(),
                            Value::DateTime(value) => value.to_string(),
                            Value::Boolean(value) => value.to_string(),
                        };
                        println!("  {}: {}", attribute.type_.label, value)
                    }
                }
                let new_user = transaction
                    .concept()
                    .get_entity_type("user".to_owned())
                    .resolve()?
                    .unwrap()
                    .create(&transaction)
                    .resolve()?;
                let _ = new_user.delete(&transaction).resolve();
            }
        }
        // end::data-api[]
    }

    {
        // tag::explain-get[]
        let db = databases.get(DB_NAME)?;
        let options = Options::new().infer(true).explain(true);
        {
            let session = Session::new(db, SessionType::Data)?;
            {
                let transaction = session.transaction_with_options(TransactionType::Read, options)?;
                let get_query = "
                                match
                                $u isa user, has email $e, has name $n;
                                $e contains 'Alice';
                                get
                                $u, $n;
                                ";
                let response = transaction.query().get(get_query)?;
                for (i, cmap) in response.enumerate() {
                    let ncmap = cmap.clone();
                    let name_concept = ncmap?.get("n").unwrap().clone();
                    let name = match name_concept {
                        Concept::Attribute(Attribute { value: Value::String(value), .. }) => value,
                        _ => unreachable!(),
                    };
                    println!("Name #{}: {}", (i + 1).to_string(), name);
                    let explainable_relations = cmap?.explainables.relations;
                    for (var, explainable) in explainable_relations {
                        println!("{}", var);
                        println!("{}", explainable.conjunction);
                        let explain_iterator = transaction.query().explain(&explainable)?;
                        for explanation in explain_iterator {
                            let exp = explanation?;
                            println!("Rule: {}", exp.rule.label);
                            println!("Condition: {}", exp.rule.when.to_string());
                            println!("Conclusion: {}", exp.rule.then.to_string());
                            println!("Variable mapping:");
                            for qvar in exp.variable_mapping.keys() {
                                println!(
                                    "Query variable {} maps to the rule variable {}",
                                    *qvar,
                                    exp.variable_mapping.get(qvar).unwrap().concat().to_string()
                                );
                            }
                        }
                    }
                }
            }
        }
        // end::explain-get[]
    }
    Ok({})
}