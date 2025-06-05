%
% Majority vote
%
% In this model, n people are voting to choose one of the d decisions.
% A decision is chosen if a strict majority of people all choose the same decision.
%
% This model is made nearly infeasible by also requiring that all decisions have been chosen fewer times than the majority needed.
% Only `needle` is allowed to have enough votes needed.
%

include "globals.mzn";

int: n; % number of people
int: d; % number of decisions
int: needle; % only viable majority


int: majority = ceil((n + 1) / 2);

array[1..n] of var 1..d: vote;


% true -> decision d must have a majority
% false -> decision d may have a majority, but does not have to
array[1..d] of var bool: require_majority;

array[1..d] of int: n_ub = [n | i in 1..d];
array[1..d] of int: cover = [i | i in 1..d];

constraint forall(i in 1..d) (
    require_majority[i] -> global_cardinality_low_up(
        vote,
        cover,
        [ if i == j then majority else 0 endif | j in 1..d ],
        n_ub
    )
);
% at least one decision must have a majority
constraint exists(i in 1..d)(require_majority[i]);


% each decision can be chosen by up to (majority - 1) people
% except, decision needle, which can get any number of votes
array[1..d] of int: zero_lb = [0 | i in 1..d];
array[1..d] of int: sub_majority_ub = [if i == needle then n else majority - 1 endif | i in 1..d];
constraint global_cardinality_low_up(vote, cover, zero_lb, sub_majority_ub);

solve satisfy;

output [
    "sub_majority_ub = ", show(sub_majority_ub), "\n",
    "\n",
    "require_majority = ", show(require_majority), "\n",
    "vote = ", show(vote), "\n",
];
