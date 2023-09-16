# `nederlandskie`

Potentially, a Bluesky feed of people speaking some language while living in some other country (for example, Russian-speaking people living in Netherlands).

Heavily WIP. Doesn't work yet at all, but does read the stream of posts as they are created on Bluesky.

## Roadmap

- [x] Read stream of posts from Bluesky
- [x] Store posts in the database
- [x] Store user profiles in the database
- [x] Detect the country of residence from profile information
- [ ] Keep subscription state to not lose messages
- [x] Serve the feed
- [ ] Publish the feed
- [ ] Handle deleting of posts

## Initial setup

Copy `.env.example` into `.env` and set up the environment variables within:

- `CHAT_GPT_API_KEY` for your ChatGPT key
- `DATABASE_URL` for PostgreSQL credentials
- `HOSTNAME` to the hostname of where you intend to host the feed

## Running

`cargo run`
