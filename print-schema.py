#!/usr/bin/env python


import psycopg
from psycopg import Cursor
from psycopg.rows import TupleRow

DB_NAME = "pissfan-testing"
DB_USER = "please"


QUERY_TABLES = """
SELECT tablename
FROM pg_catalog.pg_tables
WHERE schemaname = 'public';
"""

COLUMNS = ["column_name", "column_default", "data_type", "is_nullable"]


def table_schema(cur: Cursor[TupleRow], t: str):
    query = f"""
    SELECT 
        column_name,
        column_default,
        data_type,
        is_nullable
    FROM INFORMATION_SCHEMA.COLUMNS
    WHERE table_name = '{t}'; 
    """

    return cur.execute(query).fetchall()


conn = psycopg.connect(f"dbname={DB_NAME} user={DB_USER}")
cur = conn.cursor()

tables_query = cur.execute(QUERY_TABLES).fetchall()
table_names = [*map(lambda t: t[0], tables_query)]

schema = []

for t in table_names:
    print("\n====\nTABLE:", t, "\n====\n")
    cols_raw = list(table_schema(cur, t))

    for i, col in enumerate(cols_raw):
        col_list = list(col)
        for n, c in enumerate(COLUMNS):
            print(f"{c}\t--\t{col_list[n]}")

        print()
    print()
