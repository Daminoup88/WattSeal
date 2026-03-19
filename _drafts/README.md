# How to Use Drafts

Drafts are posts-in-progress that won't be published to your site.

## Creating a Draft

1. Create a new `.md` file in this `_drafts/` folder
2. Name it **without** a date prefix: `my-draft-title.md`
3. Add front matter at the top:

```yaml
---
layout: post
title: "Your Draft Title"
author: "Your Name"
description: "A short description for SEO and social sharing."
---
```

4. Write your content in Markdown below the front matter

## Previewing Drafts Locally

Run Jekyll with the `--drafts` flag:

```sh
bundle exec jekyll serve --drafts
```

Your drafts will appear as posts dated with today's date.

## Publishing a Draft

To publish, move the file from `_drafts/` to `_posts/`

## Front Matter Reference

| Field         | Required | Description                                      |
|---------------|----------|--------------------------------------------------|
| `layout`      | Yes      | Always `post`                                    |
| `title`       | Yes      | Post title (used in `<title>` and `<h1>`)        |
| `date`        | Yes*     | `YYYY-MM-DD`                                     |
| `author`      | No       | Author name displayed below the title            |
| `description` | No       | Used for meta description and social sharing     |
