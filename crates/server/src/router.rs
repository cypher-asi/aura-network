use axum::routing::{delete, get, post, put};
use axum::Router;

use crate::handlers;
use crate::state::AppState;

pub fn create_router() -> Router<AppState> {
    Router::new()
        // Health
        .route("/health", get(handlers::health))
        // Users
        .route(
            "/api/users/me",
            get(handlers::users::get_me).put(handlers::users::update_me),
        )
        .route("/api/users/:id", get(handlers::users::get_user))
        .route(
            "/api/users/:id/profile",
            get(handlers::users::get_user_profile),
        )
        // Profiles
        .route("/api/profiles/:id", get(handlers::users::get_profile))
        .route(
            "/api/profiles/:id/posts",
            get(handlers::feed::get_profile_activity),
        )
        // Agent profile lookup
        .route(
            "/api/agents/:id/profile",
            get(handlers::users::get_agent_profile),
        )
        // Organizations
        .route(
            "/api/orgs",
            post(handlers::orgs::create_org).get(handlers::orgs::list_orgs),
        )
        .route(
            "/api/orgs/:id",
            get(handlers::orgs::get_org)
                .put(handlers::orgs::update_org)
                .delete(handlers::orgs::delete_org),
        )
        .route("/api/orgs/:id/members", get(handlers::orgs::list_members))
        .route(
            "/api/orgs/:id/members/:userId",
            put(handlers::orgs::update_member).delete(handlers::orgs::remove_member),
        )
        .route(
            "/api/orgs/:id/invites",
            post(handlers::orgs::create_invite).get(handlers::orgs::list_invites),
        )
        .route(
            "/api/orgs/:id/invites/:inviteId",
            delete(handlers::orgs::revoke_invite),
        )
        .route(
            "/api/invites/:token/accept",
            post(handlers::orgs::accept_invite),
        )
        // Agents
        .route(
            "/api/agents",
            post(handlers::agents::create_agent).get(handlers::agents::list_agents),
        )
        .route(
            "/api/agents/:id",
            get(handlers::agents::get_agent)
                .put(handlers::agents::update_agent)
                .delete(handlers::agents::delete_agent),
        )
        // Projects
        .route(
            "/api/projects",
            post(handlers::projects::create_project).get(handlers::projects::list_projects),
        )
        .route(
            "/api/projects/:id",
            get(handlers::projects::get_project)
                .put(handlers::projects::update_project)
                .delete(handlers::projects::delete_project),
        )
        // Feed
        .route("/api/feed", get(handlers::feed::get_feed))
        .route("/api/posts", post(handlers::feed::post_activity))
        .route("/api/posts/:id", get(handlers::feed::get_post))
        // Comments
        .route(
            "/api/posts/:eventId/comments",
            get(handlers::feed::list_comments).post(handlers::feed::create_comment),
        )
        .route("/api/comments/:id", delete(handlers::feed::delete_comment))
        // Social
        .route(
            "/api/follows",
            post(handlers::social::follow).get(handlers::social::list_following),
        )
        .route(
            "/api/follows/:profileId",
            delete(handlers::social::unfollow),
        )
        .route("/api/leaderboard", get(handlers::social::leaderboard))
        // Integrations
        .route(
            "/api/orgs/:id/integrations",
            post(handlers::integrations::create_integration)
                .get(handlers::integrations::list_integrations),
        )
        .route(
            "/api/orgs/:id/integrations/:integrationId",
            put(handlers::integrations::update_integration)
                .delete(handlers::integrations::delete_integration),
        )
        // Usage & Stats
        .route("/api/orgs/:id/usage", get(handlers::usage::get_org_usage))
        .route(
            "/api/orgs/:id/usage/members",
            get(handlers::usage::get_member_usage),
        )
        .route(
            "/api/users/me/usage",
            get(handlers::usage::get_personal_usage),
        )
        .route("/api/usage", post(handlers::usage::record_usage))
        .route("/api/stats", get(handlers::usage::get_stats))
        .route("/api/orgs/:id/budget", get(handlers::usage::check_budget))
        // Access Codes
        .route(
            "/api/access-codes/redeem",
            post(handlers::access_codes::redeem_code),
        )
        .route(
            "/api/access-codes",
            get(handlers::access_codes::get_my_code),
        )
        // Internal
        .route(
            "/internal/users/:zeroUserId",
            get(handlers::internal::get_user_by_zero_id),
        )
        .route("/internal/posts", post(handlers::internal::post_activity))
        .route("/internal/usage", post(handlers::internal::record_usage))
        .route(
            "/internal/orgs/:id/members/:userId/budget",
            get(handlers::internal::check_budget),
        )
        .route(
            "/internal/projects/:projectId/usage",
            get(handlers::internal::get_project_usage),
        )
        .route(
            "/internal/orgs/:id/usage",
            get(handlers::internal::get_org_usage),
        )
        .route(
            "/internal/usage/network",
            get(handlers::internal::get_network_usage),
        )
        .route(
            "/internal/orgs/:id/integrations",
            get(handlers::internal::list_org_integrations),
        )
        // WebSocket
        .route("/ws/events", get(handlers::ws::ws_events))
}
