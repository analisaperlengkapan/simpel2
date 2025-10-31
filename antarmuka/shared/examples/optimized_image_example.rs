//! Example usage of OptimizedImage component
//!
//! This file demonstrates how to use the OptimizedImage component
//! in your microfrontend applications.

use leptos::prelude::*;
use shared_microfrontend::components::{Avatar, AvatarSize, OptimizedImage};
use shared_microfrontend::utils::{get_optimized_image_url, preload_image};

/// Example: Basic image with lazy loading
#[component]
pub fn BasicImageExample() -> impl IntoView {
    view! {
        <OptimizedImage
            src="/images/document.jpg"
            alt="Document preview"
            lazy=true
        />
    }
}

/// Example: Responsive image with multiple sizes
#[component]
pub fn ResponsiveImageExample() -> impl IntoView {
    view! {
        <OptimizedImage
            src="/images/hero.jpg"
            alt="Hero banner"
            lazy=false  // Don't lazy load hero images
            responsive=true
            sizes=vec![320, 640, 960, 1280, 1920]
            class="w-full h-64"
            object_fit="cover"
        />
    }
}

/// Example: Image gallery with lazy loading
#[component]
pub fn ImageGallery() -> impl IntoView {
    let images = vec![
        ("/images/photo1.jpg", "Photo 1"),
        ("/images/photo2.jpg", "Photo 2"),
        ("/images/photo3.jpg", "Photo 3"),
        ("/images/photo4.jpg", "Photo 4"),
    ];

    view! {
        <div class="grid grid-cols-2 md:grid-cols-4 gap-4">
            {images.into_iter().map(|(src, alt)| {
                view! {
                    <OptimizedImage
                        src=src.to_string()
                        alt=alt.to_string()
                        lazy=true
                        responsive=true
                        class="rounded-lg shadow-md"
                        object_fit="cover"
                        width=300
                        height=200
                    />
                }
            }).collect_view()}
        </div>
    }
}

/// Example: User avatar with fallback
#[component]
pub fn UserAvatarExample() -> impl IntoView {
    let user_name = "Ahmad Wijaya";
    let avatar_url = Some("/images/users/ahmad.jpg".to_string());

    view! {
        <div class="flex items-center space-x-4">
            <Avatar
                src=avatar_url
                name=user_name.to_string()
                size=AvatarSize::Large
            />
            <div>
                <h3 class="font-semibold">{user_name}</h3>
                <p class="text-sm text-gray-600">"Jaksa Agung Muda"</p>
            </div>
        </div>
    }
}

/// Example: Preloading critical images
#[component]
pub fn AppWithPreloadedImages() -> impl IntoView {
    // Preload critical images on mount
    #[cfg(target_arch = "wasm32")]
    {
        use leptos::prelude::Effect;

        Effect::new(move |_| {
            // Preload logo
            preload_image("/assets/logo.png");

            // Preload hero image
            preload_image("/assets/hero.jpg");

            // Preload common icons
            preload_image("/assets/icons/dashboard.svg");
            preload_image("/assets/icons/apps.svg");
        });
    }

    view! {
        <div>
            <img src="/assets/logo.png" alt="Logo" class="h-12" />
            <OptimizedImage
                src="/assets/hero.jpg"
                alt="Hero"
                lazy=false
                class="w-full h-96"
            />
        </div>
    }
}

/// Example: WebP format detection
#[component]
pub fn WebPExample() -> impl IntoView {
    let image_url = get_optimized_image_url("/images/photo.jpg");
    // Returns "/images/photo.webp" if supported, otherwise "/images/photo.jpg"

    view! {
        <img src=image_url alt="Optimized photo" />
    }
}

/// Example: Product card with optimized image
#[component]
pub fn ProductCard(
    #[prop(into)] name: String,
    #[prop(into)] image: String,
    #[prop(into)] description: String,
) -> impl IntoView {
    view! {
        <div class="bg-white rounded-lg shadow-md overflow-hidden">
            <OptimizedImage
                src=image
                alt=name.clone()
                lazy=true
                responsive=true
                class="w-full h-48"
                object_fit="cover"
            />
            <div class="p-4">
                <h3 class="font-semibold text-lg mb-2">{name}</h3>
                <p class="text-gray-600 text-sm">{description}</p>
            </div>
        </div>
    }
}

/// Example: Document preview with loading state
#[component]
pub fn DocumentPreview(
    #[prop(into)] document_url: String,
    #[prop(into)] title: String,
) -> impl IntoView {
    view! {
        <div class="border rounded-lg p-4">
            <h4 class="font-medium mb-2">{title}</h4>
            <OptimizedImage
                src=document_url
                alt=format!("Preview of {}", title)
                lazy=true
                class="w-full border rounded"
                object_fit="contain"
                width=800
                height=600
            />
        </div>
    }
}

/// Example: Team member list with avatars
#[component]
pub fn TeamMemberList() -> impl IntoView {
    let team_members = vec![
        ("Ahmad Wijaya", Some("/images/team/ahmad.jpg")),
        ("Siti Nurhaliza", Some("/images/team/siti.jpg")),
        ("Budi Santoso", None), // No image, will show initials
        ("Dewi Lestari", Some("/images/team/dewi.jpg")),
    ];

    view! {
        <div class="grid grid-cols-2 md:grid-cols-4 gap-6">
            {team_members.into_iter().map(|(name, avatar)| {
                view! {
                    <div class="text-center">
                        <Avatar
                            src=avatar.map(|s| s.to_string())
                            name=name.to_string()
                            size=AvatarSize::XLarge
                            class="mx-auto mb-2"
                        />
                        <p class="font-medium">{name}</p>
                    </div>
                }
            }).collect_view()}
        </div>
    }
}

/// Example: Full page with optimized assets
#[component]
pub fn OptimizedPage() -> impl IntoView {
    // Initialize asset optimization
    #[cfg(target_arch = "wasm32")]
    {
        use leptos::prelude::Effect;
        use shared_microfrontend::utils::{FontLoadingStrategy, FontType, preload_font};

        Effect::new(move |_| {
            // Apply font loading strategy
            let font_strategy = FontLoadingStrategy::default();
            font_strategy.apply();

            // Preload critical fonts
            preload_font("/fonts/inter-regular.woff2", FontType::Woff2);
            preload_font("/fonts/inter-medium.woff2", FontType::Woff2);

            // Preload critical images
            preload_image("/assets/logo.png");
            preload_image("/assets/hero.jpg");
        });
    }

    view! {
        <div class="min-h-screen bg-gray-50">
            // Header with logo
            <header class="bg-white shadow">
                <div class="container mx-auto px-4 py-4">
                    <img src="/assets/logo.png" alt="Logo" class="h-12" />
                </div>
            </header>

            // Hero section with optimized image
            <section class="relative">
                <OptimizedImage
                    src="/assets/hero.jpg"
                    alt="Hero banner"
                    lazy=false
                    responsive=true
                    class="w-full h-96"
                    object_fit="cover"
                />
                <div class="absolute inset-0 bg-black bg-opacity-50 flex items-center justify-center">
                    <h1 class="text-4xl font-bold text-white">"Welcome to SIMPelv2"</h1>
                </div>
            </section>

            // Content with lazy-loaded images
            <section class="container mx-auto px-4 py-12">
                <ImageGallery />
            </section>

            // Team section
            <section class="container mx-auto px-4 py-12">
                <h2 class="text-2xl font-bold mb-6">"Our Team"</h2>
                <TeamMemberList />
            </section>
        </div>
    }
}
