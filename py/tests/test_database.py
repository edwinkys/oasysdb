from oasysdb.database import Database
from oasysdb.collection import Record, Collection, Config


NAME = "vectors"  # Initial collection name.
DIMENSION = 128
LEN = 100


def create_test_database(path: str) -> Database:
    """Creates a new test database with an initial collection."""

    db = Database.new(path)
    assert db.is_empty()

    # Create a test collection with random records.
    records = Record.many_random(dimension=DIMENSION, len=LEN)
    db.create_collection(name=NAME, records=records)
    assert not db.is_empty()

    return db


def test_open_database():
    db = Database(path="data/101")
    assert db.is_empty()


def test_new_database():
    db = create_test_database(path="data/102")
    assert not db.is_empty()
    assert db.len() == 1


def test_get_collection():
    db = create_test_database(path="data/103")
    collection = db.get_collection(name=NAME)
    assert collection.len() == LEN


def test_save_collection():
    db = create_test_database(path="data/104")

    # Create a new collection and save it to the database.
    config = Config.create_default()
    collection = Collection(config=config)
    db.save_collection(name="test", collection=collection)

    assert db.len() == 2


def test_delete_collection():
    db = create_test_database(path="data/105")
    db.delete_collection(name=NAME)
    assert db.is_empty()
