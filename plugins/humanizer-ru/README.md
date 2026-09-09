# humanizer-ru

Removes AI-generation markers from Russian text and makes it read like a human wrote it. The skill is a Russian-language counterpart to `human-writing`: it works only with Russian and routes English text to the English skill.

## Skills

### `humanizer-ru`

Catalog of 64 AI-text patterns across 14 categories (bureaucratic phrasing, calques from English, em-dash overuse, rule of three, hollow intensifiers, rhythm flatness and others) plus 21 hard bans. Three modes: full rewrite, audit only, targeted fix. Keeps source facts locked (numbers, dates, names, links are carried over unchanged) and calibrates the voice to the author's samples.

A deterministic scanner ships with the skill (`skills/humanizer-ru/scripts/scan.py`): it scores a text 0-100, lists the markers it found and compares before/after versions. It is optional; the skill works from the catalog alone when Python or the `razdel` and `pymorphy3` packages are not available.

Thresholds were calibrated on 55K labeled Russian texts (human vs. generated).

## Install

```bash
droid plugin install humanizer-ru@factory-plugins
```

## Source and license

Upstream repository: https://github.com/ilyautov/humanizer-ru (MIT). Site with examples and an online audit: https://humanizer-ru.aifrontier.tech/
