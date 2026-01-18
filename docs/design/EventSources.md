# Event Sources for Scraping

This document tracks potential data sources for the Locate918 event aggregator.

**Owner:** Skylar (Data Engineer)  
**Last Updated:** January 2026

---

## Priority Levels
- **P1** — Must have for MVP (pick 2-3)
- **P2** — Nice to have
- **P3** — Future/stretch goal

---

## Music Venues - Large

| Name | URL | Capacity | Notes |
|------|-----|----------|-------|
| BOK Center | https://www.bokcenter.com | 19,199 | Major concerts, sports, events |
| Tulsa Theater | https://tulsatheater.com | 2,800 | Historic venue (formerly Brady Theater) |
| Tulsa PAC | https://tulsapac.com | 2,365 | Broadway, symphony, ballet |
| Mabee Center | https://www.mabeecenter.com | 10,575 | ORU campus, concerts |

## Music Venues - Medium

| Name | URL | Capacity | Notes |
|------|-----|----------|-------|
| Cain's Ballroom | https://www.cainsballroom.com | 1,800 | Historic, "Home of Bob Wills" |
| The Vanguard | https://www.thevanguardtulsa.com | 450 | Emerging artists, indie |
| Oklahoma Jazz Hall of Fame | https://jazzdepotlive.com | ~500 | Jazz, blues, gospel |
| Paradise Cove at River Spirit | https://www.riverspirittulsa.com | 2,500 | Casino venue |

## Music Venues - Small / Bars

| Name | URL | Capacity | Notes |
|------|-----|----------|-------|
| Mercury Lounge | https://www.mercuryloungetulsa.com | ~200 | Converted gas station, live music 7 nights/week |
| The Shrine | | ~150 | Dive bar, local music, low prices |
| The Colony | https://www.thecolonytulsa.com | ~200 | Local bands, intimate setting |
| The Fur Shop | | ~150 | Taproom + concert venue, craft beer |
| Soundpony | | ~100 | Dive bar, DJ nights |
| LowDown | | ~150 | Jazz club, Arts District |
| Maggie's Music Box | https://www.maggiesmusicbox.com | ~150 | Jenks, gastropub + live music |
| The Yeti | | ~100 | Bar with live local bands |

## Theaters / Performing Arts

| Name | URL | Notes |
|------|-----|-------|
| Tulsa PAC - Chapman Music Hall | https://tulsapac.com | 2,365 seats, Broadway tours |
| Tulsa PAC - Williams Theatre | https://tulsapac.com | 430 seats |
| Tulsa PAC - Doenges Theatre | https://tulsapac.com | 150 seats |
| Circle Cinema | https://www.circlecinema.org | Art house films, special events |
| Tulsa Ballet | https://tulsaballet.org | Performs at PAC |
| Tulsa Opera | https://tulsaopera.com | Performs at PAC |

## Sports Venues

| Name | URL | Notes |
|------|-----|-------|
| ONEOK Field | https://www.tulsadrillers.com | Tulsa Drillers (baseball) |
| BOK Center | https://www.bokcenter.com | Tulsa Oilers (hockey) |
| H.A. Chapman Stadium | | TU football |
| FC Tulsa | https://www.fctulsa.com | Soccer |

## Museums / Cultural (with events)

| Name | URL | Notes |
|------|-----|-------|
| Philbrook Museum | https://philbrook.org | Art museum, hosts events |
| Gilcrease Museum | https://gilcrease.org | Western art, events |
| Woody Guthrie Center | https://woodyguthriecenter.org | Music history, concerts |
| Bob Dylan Center | https://bobdylancenter.com | Music history, events |

## Aggregators / Platforms

| Name | URL | Notes |
|------|-----|-------|
| Eventbrite (Tulsa) | https://www.eventbrite.com/d/ok--tulsa/events | Has API - GOOD STARTING POINT |
| Visit Tulsa | https://www.visittulsa.com/events | Official tourism, clean listing |
| Bandsintown (Tulsa) | https://www.bandsintown.com/c/tulsa-ok | Concert aggregator |
| Songkick | https://www.songkick.com/metro-areas/29587-us-tulsa | Concert aggregator |
| Downtown Tulsa | https://downtowntulsa.com/events | Downtown events |
| Tulsa People | https://www.tulsapeople.com/about-town | Local magazine listings |
| Tulsa World Calendar | https://www.tulsaworld.com/calendar | Newspaper calendar |

## Community / Local

| Name | URL | Notes |
|------|-----|-------|
| Meetup (Tulsa) | https://www.meetup.com/find/?location=tulsa--ok | Tech, hobby groups |
| Gathering Place | https://www.gatheringplace.org | Park events |
| Tulsa Library | https://www.tulsalibrary.org | Free events |
| Philbrook Downtown | https://philbrook.org | Art events |

## Casinos (with event venues)

| Name | URL | Notes |
|------|-----|-------|
| River Spirit Casino | https://www.riverspirittulsa.com | Paradise Cove venue |
| Hard Rock Tulsa | https://www.hardrockcasinotulsa.com | The Joint venue |
| Osage Casino | https://www.osagecasinos.com | Skyline Event Center |

---

## Recommended Starting Points

For MVP, Skylar should prioritize these sources:

### Tier 1 (Start here)
1. **Eventbrite** - Has API, many events, structured data
2. **Visit Tulsa** - Official tourism site, clean listings
3. **Cain's Ballroom** - Well-structured website, iconic venue

### Tier 2 (Add next)
4. **Tulsa PAC** - Broadway, symphony, major events
5. **The Vanguard** - Indie/emerging artists
6. **BOK Center** - Major concerts

### Tier 3 (If time permits)
7. Mercury Lounge
8. Oklahoma Jazz Hall of Fame
9. Downtown Tulsa events

---

## Notes for Skylar

When evaluating a source, check:
1. Does it have a clean events listing page?
2. Is there an API available? (easier than scraping)
3. Does robots.txt allow scraping?
4. How often is it updated?
5. What event types does it cover?

### API Priority
- Eventbrite has a public API - USE THIS FIRST
- Bandsintown has an API
- Most venue sites require HTML scraping

### Scraping Tips
- Start with ONE source, get it working end-to-end
- Use the LLM `/api/normalize` endpoint to parse messy data
- Store `source_url` for every event - we link back to organizers
- Check for duplicates (same event on multiple sites)
