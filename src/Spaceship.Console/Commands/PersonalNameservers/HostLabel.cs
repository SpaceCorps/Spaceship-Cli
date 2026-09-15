namespace Spaceship.Console.Commands.PersonalNameservers;

internal static class HostLabel
{
    // The API addresses a personal nameserver by its label only: "ns1" for ns1.example.com.
    public static string Normalize(string host, string domain)
    {
        var label = host.Trim().TrimEnd('.');
        var suffix = "." + domain.Trim().TrimEnd('.');
        return label.EndsWith(suffix, StringComparison.OrdinalIgnoreCase) ? label[..^suffix.Length] : label;
    }
}
