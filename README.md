# `nederlandskie`

Potentially, a Bluesky feed of people speaking some language while living in some other country (for example, Russian-speaking people living in Netherlands).

Heavily WIP. Doesn't work yet at all, but does read the stream of posts as they are created on Bluesky.

## Roadmap

- [x] Read stream of posts from Bluesky
- [x] Store posts in the database
- [x] Store user profiles in the database
- [x] Detect the country of residence from profile information
- [x] Keep subscription state to not lose messages
- [x] Serve the feed
- [x] Handle deleting of posts
- [ ] Handle errors in the web service gracefully
- [ ] Handle missing profiles in the profile classifier
- [ ] Add a way to mark a profile as being from a certain country manually
- [ ] Handle reconnecting to websocket somehow
- [ ] Publish the feed

## Configuration

1. Copy `.env.example` into `.env` and set up the environment variables within:

   - `PUBLISHER_BLUESKY_HANDLE` to your Bluesky handle
   - `PUBLISHER_BLUESKY_PASSWORD` to Bluesky app password that you created in settings
   - `CHAT_GPT_API_KEY` for your ChatGPT key
   - `DATABASE_URL` for PostgreSQL credentials
   - `FEED_GENERATOR_HOSTNAME` to the hostname of where you intend to host the feed

2. Determine your own DID and put it in `PUBLISHER_DID` env variable in `.env`:

   ```
   cargo run --bin who_am_i
   ```

## Running

### Populate and serve the feed

`cargo run`

### Determine your own did for publishing

`cargo run --bin who_am_i`

### Publish the feed

`cargo run --bin publish_feed -- --help`
