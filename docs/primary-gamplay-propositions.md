# Wizard's Battle

Game has a premise similar to Battle (the card game, War in the US, but we
use Battle to avoid trademark infringement). However each draw is 3 cards and
you get to chose which card to play until the 3 cards are exhausted and you
draw again.

You play against a pigeon and a blindfolded man with a pointy hat and a eye
drawn on the blindfold.

Your card suit is not the usual suite, but a deck of magic words. Each word
has a value and a color (four color totals). There are some quirks that makes
the game more interesting than the base Battle game (like maybe a
rock/paper/scissor mechanic).

## Gameplay

As in Battle, the goal is to accumulate as many cards as possible, at the end
the magic words form a pentacle that summons a demon, and the one with the
most magic words is the largest and beats the other.

The game starts exactly like Battle... Apart that you'll soon notice your
deck is the lower half of values while your opponents is the upper half! They
just so happen to have sorted themselves the deck prior. You lose quickly and
you restart.

Now game shows a prompt that tells you to "hide" a card while "the pigeon is
not looking". You hide it, then you get a prompt to "reuse" at the next draw.
It's a sort of quicktime event so that you can store a card between turns. If
you fail to properly put back the card in your hand while drawing, you get
noticed and the pigeon use his eye lasers to kill you. *(Note by blucky: I think eye lasers would be out of place but since it's a bird an option would be to have the pigeon peck you or something, or have it fly up to reveal claws that would fit a predatory bird more than a pigeon with which it will kill you.
Another concern would be lives: should we instakill or should the pigeon cause injury first and then kill if you mess up too much?)*
*(gib: I think the pigeon should flutter and make noise to make the blindfold wizard notice you and they say something like "cheater be gone!")*
*(vasukas: Good thing about eye lasers is that no-one will expect that! And it should be faster to do than pecking animation and way cooler than just game over screen)*

Each "run" should be about 5 minutes. The first one where you lose by default should be at most 2 minutes. Each cheat mistake is fatal and you should be able to restart very quickly.

There are more than one trick. Some are classic tricks like marking, putting
back in the deck etc. Some of them include magic (like making a card
invisible or duplicating or swapping with enemy hand or picking from enemy
deck). The magic tricks can be spotted by the blindfolded man while the
physical tricks by the pigeon. You have to somehow get them to be distracted
to pull the tricks.

### Basic card game mechanic

* Each turn, the initiative is swapped between oppo and player
* At beginning of the game, player choses who has initiative.
* If hand empty, draw 3 cards from draw pile
* Turn:
    * When you have the initiative, you play one card
    * participant without initiative play their card
    * Greatest value card wins (exception: see extra rules)
    * Winner gets to add the cards to their win pile
* Ends when draw pile is empty

### Advanced card game mechanics

The cards have no suit, only color and words of magic. We could potentially add suits if we have time.

The card numbers are swapped with [Maya numerals](https://en.wikipedia.org/wiki/Maya_numerals),
from 0 to 12. Higher value cards win over lower value cards, except 0 which beats 12
(but beaten by everything else).

Each card has an enochian word of magic attached to it. There is exactly 10
words of magic (see [wikipedia](https://en.wikipedia.org/wiki/Enochian)). Words
of magic act like special modifiers on the card or can give bonus actions to player. (XO: the words are ~paragraph long prayers, probably going to just have to substitute them with like, a gylph from the Enochian language, not enough space to do intricate special text in more than like ~3 character long strings, they won't really be able to actually mean anything though)

TODO: design word of magic effects.

### Distraction mechanics

Sometimes the bird or the man are distracted. The bird often, the man rarely.

You can use seeds or maybe interact with objects on the table to distract the bird
and open a window of opportunity to do some cheats.

There is an explicit UI elements giving the player an idea how long oppo is
distracted. Each cheat has a different length of time. If you are spotted cheating,
it's game over, no appeal.

TODO: magic cheat mechanic

### Cheats

"physical" cheats are noticed by the bird, while "magic" cheats are noticed by
the man. "free" cheats are noticed by no one.

* Put card in sleeve (physical): When drawing next 3 cards, draw a fourth you put in your sleeve,
  you may retrieve it next draw by clicking your sleeve while retrieving the cards
* Swap card (physical): Very discretely swap one of the card in your hand with that of the oppo. (remember the man is blindfolded)
* Peek (physical): look at top of draw pile, can chose to swap cards in your hand with it.
* Hack platonic ordering (magic): Until next draw, lower value cards win.
* Pull out seeds (free): Get the bird distracted by giving them seeds
* Inner eye (magic): See the oppo's hand
* Look over shoulder (physical): See the oppo's hand, requires extreme distraction from the bird
* Déjà vu (magic): swap initative for this turn

