CREATE TABLE torrust_categories (
    category_id INTEGER NOT NULL PRIMARY KEY AUTO_INCREMENT,
    name VARCHAR(64) NOT NULL UNIQUE
);

INSERT INTO torrust_categories (name) VALUES ('movies'), ('tv shows'), ('games'), ('music'), ('software');
