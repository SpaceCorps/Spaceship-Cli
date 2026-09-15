using System.Text.Json;

namespace Spaceship.Console.Infrastructure;

public static class JsonInput
{
    public static async Task<JsonElement> ReadAsync(string? file, string expected)
    {
        string json;
        if (!string.IsNullOrWhiteSpace(file))
            json = await System.IO.File.ReadAllTextAsync(file);
        else if (!System.Console.IsInputRedirected)
            throw new SpaceshipException($"Provide JSON via stdin or --file. Expected: {expected}");
        else
            json = await System.Console.In.ReadToEndAsync();

        try
        {
            return JsonSerializer.Deserialize<JsonElement>(json);
        }
        catch (JsonException ex)
        {
            throw new SpaceshipException($"Invalid JSON: {ex.Message}");
        }
    }
}
