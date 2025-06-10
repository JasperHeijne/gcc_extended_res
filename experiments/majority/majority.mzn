%
% Majority vote
%
% In this model, n people are voting to choose one of the d options.
% A option is chosen if a strict majority of people all choose the same option.
%
% This model is made nearly infeasible by also requiring that all options have been chosen fewer times than the majority needed.
% Only `needle` is allowed to have enough votes needed.
% If `needle` is in 1..d, then the problem is unsatisfiable.
% Otherwise, it is satisfiable with option number `needle` having the majority.
%

include "globals.mzn";

int: n; % number of people
int: d; % number of options
int: needle; % only viable majority


int: majority = ceil((n + 1) / 2);

array[1..n] of var 1..d: vote;


% true -> option d must have a majority
% false -> option d may have a majority, but does not have to
array[1..d] of var bool: require_majority;

array[1..d] of int: n_ub = [n | i in 1..d];
array[1..d] of int: cover = [i | i in 1..d];

constraint forall(i in 1..d) (
    pumpkin_gcc_imp(
        vote,
        cover,
        [ if i == j then majority else 0 endif | j in 1..d ],
        n_ub,
        require_majority[i]
    )
);
% at least one option must have a majority
constraint exists(i in 1..d)(require_majority[i]);


% each option can be chosen by up to (majority - 1) people
% except, option needle, which can get any number of votes
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
