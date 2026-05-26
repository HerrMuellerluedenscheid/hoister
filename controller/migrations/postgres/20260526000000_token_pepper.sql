-- Token storage switched from plain SHA-256 to HMAC-SHA256(pepper, token).
-- Existing hashes are unsalted SHA-256 and cannot be re-keyed without the
-- plaintext tokens (which we never stored). Drop them; the dashboard will
-- mint a fresh token on the next login.
DELETE FROM api_token;
