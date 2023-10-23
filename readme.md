# StimkerBot

> A Telegram bot to easily organize your stickers by tags

## Why?

When you get to the point of hitting the sticker pack limit on telegram, it becoming awfully hard to find that right sticker before the conversation moves on. This bot aims to solve that by allowing you to tag your stickers and search by those tags. This way the tags are tailored to your needs and you can find the right sticker in no time.

## How to

Just send the bot a new sticker, and it will ask you for tags. After that you can search for those tags and it will return the matching stickers. You can search for stickers by mentioning the bot in your chat field, and then listing your tags.

## Usage

### Hosted

The easiest way is to just use the hosted version: [@StimkerBot](https://t.me/StimkerBot)

### Self-hosted - Docker

Use docker-compose to run the bot:

```yml
version: '3'
services:
  stimkerbot:
    container_name: stimkerbot
    image: ghcr.io/caramelfur/stimkerbot:latest
    user: '1000:1000'
    environment:
      TELOXIDE_TOKEN: 'YOUR_TG_BOT_TOKEN'
    restart: unless-stopped
    volumes:
      - ./stimkerbot:/data
```

When self hosting make sure to enable inline requests, and inline reporting.



