# Wizard's War

Game has a premise similar to War (the child card game). However each draw is 3 cards and
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
the one with the most points win. (Loser gets struck by lighting)

The game starts exactly like Battle... Each pooling from a different deck. You
soon notice that the cards are stacked: your opponent only has high values while
you only have low values. You lose quickly and you restart. Design note: it is
necessary to pool from two different decks otherwise it's boring-easy to win if
we have any cheat/effects that enable adding/removing cards from the decks.

Now game shows a prompt showing that an item on the table is interactible. You click
on it and it distracts the bird. You get the prompt to hide one card while the bird
is distracted. It's a sort of quicktime event so that you can store a card between
turns. If you fail to properly put back the card in your hand while drawing, you get
noticed and the pigeon use his eye lasers to kill you. *(Note by blucky: I think eye lasers would be out of place but since it's a bird an option would be to have the pigeon peck you or something, or have it fly up to reveal claws that would fit a predatory bird more than a pigeon with which it will kill you.
Another concern would be lives: should we instakill or should the pigeon cause injury first and then kill if you mess up too much?)*
*(gib: I think the pigeon should flutter and make noise to make the blindfold wizard notice you and they say something like "cheater be gone!")*
*(vasukas: Good thing about eye lasers is that no-one will expect that! And it should be faster to do than pecking animation and way cooler than just game over screen)*

Each "run" should be about 5 minutes. The first one where you lose by default should be at most 2 minutes. Each cheat mistake is fatal and you should be able to restart very quickly. To speed up things, it might be possible to do
compute "possible max score" and fail early if it's impossible to win.

There are more than one trick. Some are classic tricks like marking, putting
back in the deck etc. Some of them include magic (like making a card
invisible or duplicating or swapping with enemy hand or picking from enemy
deck). The magic tricks can be spotted by the blindfolded man while the
physical tricks by the pigeon. You have to somehow get them to be distracted
to pull the tricks.

### Basic card game mechanic

* There is two draw piles, one per player
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

#### War

If both cards played in a turn have the same value, there is "War", then all cards in hand are staked.
The war is resolved the same way as in "War".

#### Card description

The cards have no suit, only color and words of power. We could potentially add suits if we have time.

The card numbers are swapped with [Maya numerals](https://en.wikipedia.org/wiki/Maya_numerals),
from 0 to 12 *(gib: looks like XO decided it was 0-9 in arabic?)*. Higher value cards win over lower value cards, except 0 which beats 12
(but beaten by everything else).

Each card has an enochian word of power attached to it. (see [wikipedia](https://en.wikipedia.org/wiki/Enochian)). Words
of power act like special modifiers on the card or can give bonus actions to player.

Words of power can appear on playing cards. If you win a turn with a card with
a word of power, you gain abilities:
* Egeq: Fertility, gain seeds
* Geh: Mana, gain MP (for magic cheats)
* Het: Mischief, learn a new physical cheat
* Meb: Fog, causes the magician to be "distracted" for the length of next turn.
* Qube: Fortune, double the points gained from this turn
* Zihbm: tbd

(the name are the transliteration of the symbols Xo drew)

### Distraction mechanics

Sometimes the bird or the man are distracted. The bird often, the man rarely.

You can use seeds or interact with objects on the table to distract the bird
and open a window of opportunity to do some cheats.

There is multiple ways to distract the bird:
* With "seeds", obtaining through playing certain cards
* With interactible objects on the table. It's one-time only and there is only
  two or three such interaction
 
The magician can only be distracted through playing a `Meb` card.

There is an explicit UI elements giving the player an idea how long oppo is
distracted. Each cheat has a different length of time. If you are spotted cheating,
it's game over, no appeal.


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

