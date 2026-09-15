using System.Globalization;

namespace Spaceship.Console.Infrastructure;

public static class QueryString
{
    // Skips null and blank values, so optional flags can be passed straight through.
    public static string Build(string path, params (string Key, object? Value)[] parameters)
    {
        var parts = parameters
            .Where(p => p.Value is not null && !(p.Value is string s && string.IsNullOrWhiteSpace(s)))
            .Select(p => $"{Uri.EscapeDataString(p.Key)}={Uri.EscapeDataString(Convert.ToString(p.Value, CultureInfo.InvariantCulture)!)}");
        var query = string.Join("&", parts);
        return query.Length == 0 ? path : $"{path}?{query}";
    }
}
