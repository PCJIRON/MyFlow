# MyFlow — Design Glossary

*Stack-agnostic tokens. Raw values pulled from the three pages in `ui/`. The pages are plain CSS, so these are copied, not generated.*

## Colour

```
dark page          #0b0b0d            settings window body
card surface       rgba(255,255,255,.032)
card line          rgba(255,255,255,.075)
pill surface       rgba(13,13,16,.97)
pill resting fill  #000000
pill recording     #0a0a0c
notes surface      rgba(19,19,23,.94)
notes line         rgba(255,255,255,.09)
text               #ececf2
text dim           #8b8b98
text soft          #b6b6c2
text faint         #6f6f7b
accent (record)    #e35244
error surface      rgba(38,15,18,.97)
error line         rgba(227,82,68,.5)
error text         #ffb4ad
switch off track   #2b2b33
switch thumb       #8b8b98
```

Hover and active states are white with low alpha, never a second brand colour:

```
hover fill         rgba(255,255,255,.05)   → .12 on stronger controls
selected fill      rgba(255,255,255,.10)
selected border    rgba(255,255,255,.22)
focus border       rgba(255,255,255,.28)
```

## Radius

```
pill / capsule     999px
card               16px
notes shell        16px
segment button     12px
input / select     10px
icon plate         13px
small icon button  9px
toast              999px
```

## Spacing

```
page padding       20px 22px 28px
card padding       16px 18px
card gap           12px
control gap        7px   (segments)
row gap            12px
pill inner padding 0 7px (open), 0 14px (recording)
```

## Typography

```
family      "Segoe UI Variable Display", "Segoe UI", sans-serif
mono        ui-monospace, "Cascadia Mono", Consolas
page base   13px
card label  12px, colour #b6b6c2
section h2  10.5px, uppercase, letter-spacing .1em, colour dim
h1          16.5px / 600
hint        11px, colour #6f6f7b, line-height 1.45
segment     12.5px / 600, sub-label 9.5px / 500, opacity .7
pill error  11px, line-height 1.4
```

## Size

```
settings window     600 x 840 (resizable)
notes window        400 x 480 (resizable)
pill window         idle 54x10 · open 402x56 · rec 130x32   (bottom dock)
pill window (side)  idle 10x54 · open 56x402 · rec 32x130
pill bar            idle 52x8 · open 100% x 54 · rec 128x30
pill bar (side)     idle 8x52 · open 54x212 · rec 30x128
knob                42x42, icon 19x19
switch              40x23, thumb 17x17, travel 17px
```

## Elevation and material

```
shadow        none — everywhere. No drop shadows on pill, cards or buttons.
blur (pill)   backdrop-filter: blur(22px)
blur (notes)  backdrop-filter: blur(18px)
blur (toast)  backdrop-filter: blur(14px)
borders       one hairline in a white-alpha value; the only visual separation
```

## Motion

```
pill shape      .28s cubic-bezier(.22,.9,.24,1)   width, height, padding
pill colour     .22s ease
knob opacity    .16s ease, transform .2s ease (scale .9 → 1)
button hover    .15s
switch          .18s
toast           .2s opacity + translateY(6px)
error box       .25s opacity
waveform        .9s base, per-bar 1.15s / .75s / 1.35s / .6s with negative delays
```

## Motion rules

- *The pill never slides into place. It snaps to an edge and only its size animates.*
- *Shrinking waits 320ms so the capsule can finish collapsing inside the window before the window itself gets small.*
- *Hovering scales nothing; only opacity and width change. The bar must stay crisp while it grows.*

## Icons

Inline SVG only, 24x24 viewBox, `stroke: currentColor`, `stroke-width: 1.8`, round caps and joins, no fills except the recording dot. No icon font, no image files.
