CREATE TABLE user (
    uuid CHAR(36),
    username TEXT NOT NULL,

    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    deleted_at TIMESTAMP,

    PRIMARY KEY(uuid)
);

CREATE TABLE public_key (
    `fingerprint`   CHAR(47),
    `user`          CHAR(36) NOT NULL,

    PRIMARY KEY(fingerprint),
    FOREIGN KEY(user) REFERENCES user(uuid)
);

INSERT INTO user (uuid, username, deleted_at) VALUES('ceef801c-ee91-491d-93ce-e7682d12fa78', 'guochao', NULL);
INSERT INTO public_key(fingerprint, user) VALUES('b0:9e:ac:48:63:fa:8a:71:b3:c3:8d:96:08:9b:fe:85', 'ceef801c-ee91-491d-93ce-e7682d12fa78');
INSERT INTO public_key(fingerprint, user) VALUES('d1:01:7e:d1:5e:16:9c:af:d2:20:eb:33:26:8f:98:f8', 'ceef801c-ee91-491d-93ce-e7682d12fa78');