"""Realistic queries for the public GitHub and SpaceX GraphQL schemas.

These queries are not toy examples -- they mirror the kind of operations real
clients send (relay-style pagination, fragments, variables, directives) so the
benchmark numbers reflect production-shaped workloads rather than the synthetic
documents in :mod:`queries`.
"""

from __future__ import annotations

# ---------------------------------------------------------------------- GitHub

GITHUB_SIMPLE = """
query Viewer {
  viewer { login name }
}
"""

GITHUB_MEDIUM = """
query MyRepos($first: Int!) {
  viewer {
    login
    repositories(first: $first, orderBy: {field: UPDATED_AT, direction: DESC}) {
      totalCount
      nodes {
        id
        name
        nameWithOwner
        url
        stargazerCount
        forkCount
        primaryLanguage { name color }
        description
      }
    }
  }
}
"""

GITHUB_COMPLEX = """
query PRExplorer(
  $orgs: Int!
  $reposPerOrg: Int!
  $prsPerRepo: Int!
  $reviewsPerPR: Int!
  $withReviews: Boolean!
) {
  viewer {
    login
    ...UserBrief
    organizations(first: $orgs) {
      nodes {
        id
        login
        name
        repositories(first: $reposPerOrg, orderBy: {field: UPDATED_AT, direction: DESC}) {
          totalCount
          nodes {
            id
            name
            nameWithOwner
            url
            stargazerCount
            primaryLanguage { name }
            defaultBranchRef { name }
            pullRequests(first: $prsPerRepo, states: [OPEN]) {
              totalCount
              nodes {
                id
                number
                title
                url
                createdAt
                updatedAt
                author { login ... on User { name } }
                ...PRBrief
                reviews(first: $reviewsPerPR) @include(if: $withReviews) {
                  nodes {
                    id
                    state
                    body
                    author { login }
                    submittedAt
                  }
                }
                comments(first: 5) {
                  nodes {
                    id
                    body
                    author { login }
                    createdAt
                  }
                }
                labels(first: 10) {
                  nodes { name color }
                }
                files(first: 10) {
                  nodes {
                    path
                    additions
                    deletions
                  }
                }
                commits(first: 5) {
                  nodes {
                    commit {
                      oid
                      messageHeadline
                      author { name email date }
                      additions
                      deletions
                    }
                  }
                }
              }
            }
          }
        }
      }
    }
  }
}

fragment UserBrief on User {
  id
  login
  name
  avatarUrl
  bio
  url
}

fragment PRBrief on PullRequest {
  baseRefName
  headRefName
  isDraft
  mergeable
  state
  changedFiles
  additions
  deletions
}
"""

# ---------------------------------------------------------------------- SpaceX

SPACEX_SIMPLE = """
query Capsules {
  capsules(limit: 1) { id }
}
"""

SPACEX_MEDIUM = """
query Launches($limit: Int!) {
  launches(limit: $limit) {
    id
    mission_name
    launch_date_utc
    rocket {
      rocket_name
      rocket_type
    }
    launch_site {
      site_name
      site_name_long
    }
    links {
      article_link
      video_link
      mission_patch
    }
  }
}
"""

SPACEX_COMPLEX = """
query LaunchExplorer($limit: Int!, $withDetails: Boolean!) {
  launches(limit: $limit) {
    ...LaunchBrief
    ships @include(if: $withDetails) {
      id
      name
      type
      home_port
      missions { name flight }
    }
    rocket {
      rocket_name
      rocket {
        ...RocketDetails
      }
      first_stage {
        cores {
          flight
          gridfins
          legs
          reused
          land_success
          landing_intent
          landing_type
        }
      }
      second_stage {
        block
        payloads {
          payload_type
          payload_mass_kg
          customers
          orbit
          nationality
        }
      }
    }
    telemetry { flight_club }
    links {
      article_link
      mission_patch
      mission_patch_small
      reddit_campaign
      reddit_launch
      video_link
    }
  }
}

fragment LaunchBrief on Launch {
  id
  mission_name
  launch_date_utc
  launch_success
  launch_year
  details
}

fragment RocketDetails on Rocket {
  id
  name
  type
  active
  description
  cost_per_launch
  success_rate_pct
  height { meters feet }
  diameter { meters feet }
  mass { kg lb }
}
"""

# ---------------------------------------------------------------------- index

QUERIES_BY_SCHEMA: dict[str, dict[str, str]] = {
    "github.graphql": {
        "simple": GITHUB_SIMPLE,
        "medium": GITHUB_MEDIUM,
        "complex": GITHUB_COMPLEX,
    },
    "spacex.graphql": {
        "simple": SPACEX_SIMPLE,
        "medium": SPACEX_MEDIUM,
        "complex": SPACEX_COMPLEX,
    },
}

VARIABLES_BY_QUERY: dict[tuple[str, str], dict[str, object]] = {
    ("github.graphql", "medium"): {"first": 10},
    ("github.graphql", "complex"): {
        "orgs": 5,
        "reposPerOrg": 5,
        "prsPerRepo": 10,
        "reviewsPerPR": 5,
        "withReviews": True,
    },
    ("spacex.graphql", "medium"): {"limit": 10},
    ("spacex.graphql", "complex"): {"limit": 5, "withDetails": True},
}


__all__ = ("QUERIES_BY_SCHEMA", "VARIABLES_BY_QUERY")
