<?php

declare(strict_types=1);

namespace Tests\Feature;

use App\Models\Pengguna\Pengguna;
use App\Services\AIService;
use Mockery;
use Tests\TestCase;

class AIControllerTest extends TestCase
{
    public function test_ask_endpoint_returns_valid_answer()
    {
        $mock = Mockery::mock(AIService::class);
        $mock->shouldReceive('answerQuestion')
            ->once()
            ->andReturn([
                'answer' => 'DEFINISI: BMN\nPENJELASAN: BMN adalah aset negara.',
                'source' => 'ai',
                'log' => [],
            ]);
        $this->app->instance(AIService::class, $mock);
        $user = Pengguna::first() ?: Pengguna::factory()->create();
        $this->actingAs($user);
        $response = $this->postJson('/ai/chat', [
            'prompt' => 'Apa itu BMN?',
        ]);
        $response->assertStatus(200)
            ->assertJson([
                'response' => 'DEFINISI: BMN\nPENJELASAN: BMN adalah aset negara.',
                'source' => 'ai',
            ]);
    }

    public function test_ask_endpoint_validation_fails()
    {
        $mock = \Mockery::mock(\App\Services\AIService::class);
        $this->app->instance(\App\Services\AIService::class, $mock);
        $user = Pengguna::first() ?: Pengguna::factory()->create();
        $this->actingAs($user);
        $response = $this->postJson('/ai/chat', [
            'prompt' => '',
        ]);
        $response->assertStatus(422)
            ->assertJsonValidationErrors(['prompt']);
    }

    public function test_ask_endpoint_handles_exception()
    {
        $mock = \Mockery::mock(\App\Services\AIService::class);
        $mock->shouldReceive('answerQuestion')
            ->once()
            ->andThrow(new \Exception('AI error'));
        $this->app->instance(\App\Services\AIService::class, $mock);
        $user = Pengguna::first() ?: Pengguna::factory()->create();
        $this->actingAs($user);
        $response = $this->postJson('/ai/chat', [
            'prompt' => 'Apa itu BMN?',
        ]);
        $response->assertStatus(500)
            ->assertJsonFragment([
                'message' => 'AI error',
            ]);
    }
}
