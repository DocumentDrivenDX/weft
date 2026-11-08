\set ON_ERROR_STOP on
CREATE SCHEMA plugin_demo;
CREATE TABLE plugin_demo.notes (id text COLLATE pg_catalog."C" PRIMARY KEY, owner text COLLATE pg_catalog."C" NOT NULL);
INSERT INTO plugin_demo.notes VALUES ('a','A'),('b','B'),('c','C'),('d','D');
PREPARE filtered_page(text,text,text,text,text,int8) AS
WITH "__weft_source" AS MATERIALIZED (
  SELECT ROW_NUMBER() OVER (ORDER BY "r"."id" COLLATE pg_catalog."C" ASC) AS "__weft_ordinal",
         "r"."id" COLLATE pg_catalog."C" AS "__weft_page_key",
         "r"."owner" COLLATE pg_catalog."C" AS "__weft_field_1"
  FROM "plugin_demo"."notes" AS "r"
  WHERE ("r"."owner" COLLATE pg_catalog."C" = $1::pg_catalog.text AND
        ("r"."owner" COLLATE pg_catalog."C" = $2::pg_catalog.text OR "r"."owner" COLLATE pg_catalog."C" = $3::pg_catalog.text))
    AND ($4::pg_catalog.text IS NULL OR "r"."id" COLLATE pg_catalog."C" > $5::pg_catalog.text COLLATE pg_catalog."C")
  ORDER BY "r"."id" COLLATE pg_catalog."C" ASC LIMIT ($6::pg_catalog.int8 + 1)
), "__weft_guard" AS MATERIALIZED (
 SELECT CASE WHEN COALESCE(pg_catalog.sum(pg_catalog.octet_length(pg_catalog.convert_to(pg_catalog.row_to_json(f)::pg_catalog.text, 'UTF8'))),0) <= 8388608
 THEN 1::pg_catalog.int8 ELSE 1::pg_catalog.int8 / (pg_catalog.count(*) - pg_catalog.count(*)) END AS ok
 FROM "__weft_source" AS f
)
SELECT f.* FROM "__weft_source" AS f ORDER BY f."__weft_ordinal" LIMIT ALL OFFSET (SELECT 1 - ok FROM "__weft_guard");
EXECUTE filtered_page('B','B','C',NULL,NULL,10);
DEALLOCATE filtered_page;
