%
% Majority vote
%
% In this model, n people are voting to choose one of the d decisions.
% A decision is chosen if a strict majority of people all choose the same decision.
%
% This model is made infeasible by also requiring that all decisions have been chosen fewer times than the majority needed.
%

include "globals.mzn";

int: n; % number of people
int: d; % number of decisions

int: majority = ceil((n + 1) / 2);

array[1..n] of var 1..d: choice;


% true -> decision d must have a majority
% false -> decision d may have a majority, but does not have to
array[1..d] of var bool: require_majority;

array[1..d] of int: ub = [n | i in 1..d];
array[1..d] of int: cover = [i | i in 1..d];

constraint forall(i in 1..d) (
    require_majority[i] -> global_cardinality_low_up(
        choice,
        cover,
        [ if i == j then majority else 0 endif | j in 1..d ],
        ub
    )
);
% at least one decision must have a majority
constraint exists(i in 1..n)(require_majority[i]);


% each decision can be chosen by up to (majority - 1) people
array[1..d] of int: zero_lb = [0 | i in 1..d];
array[1..d] of int: sub_majority_ub = [majority - 1 | i in 1..d];
constraint global_cardinality_low_up(choice, cover, zero_lb, sub_majority_ub);

solve satisfy;
