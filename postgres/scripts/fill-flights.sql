CREATE TABLE airport
(
    id      SERIAL PRIMARY KEY,
    name    VARCHAR(255),
    city    VARCHAR(255),
    country VARCHAR(255)
);

CREATE TABLE flight
(
    id              SERIAL PRIMARY KEY,
    flight_number   VARCHAR(20)              NOT NULL,
    datetime        TIMESTAMP WITH TIME ZONE NOT NULL,
    from_airport_id INT REFERENCES airport (id),
    to_airport_id   INT REFERENCES airport (id),
    price           INT                      NOT NULL
);

INSERT INTO airport(name, city, country) VALUES ('Шереметьево', 'Москва', 'Россия');
INSERT INTO airport(name, city, country) VALUES ('Пулково', 'Санкт-Петербург', 'Россия');
INSERT INTO flight(flight_number, datetime, from_airport_id, to_airport_id, price) VALUES ('AFL031', '2021-10-08 23:00:00.000 +0300', 2, 1, 1500);
