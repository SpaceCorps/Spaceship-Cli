using System.ComponentModel;
using Spaceship.Console.Infrastructure;
using Spectre.Console.Cli;

namespace Spaceship.Console.Commands.SellerHub.Safepay;

[Description("List SafePay transactions you are a party to")]
public sealed class ListCommand : SpaceshipCommand<PaginatedSettings>
{
    protected override async Task<object> ExecuteAsync(SpaceshipApiClient client, PaginatedSettings settings)
    {
        var result = await client.GetAsync(QueryString.Build("/sellerhub/safepay-transactions",
            ("take", settings.Take), ("skip", settings.Skip)));
        return ToObject(result);
    }
}

public sealed class GetSettings : GlobalSettings
{
    [CommandArgument(0, "<transaction-id>")]
    [Description("SafePay transaction ID")]
    public required string TransactionId { get; set; }
}

[Description("Get a SafePay transaction")]
public sealed class GetCommand : SpaceshipCommand<GetSettings>
{
    protected override async Task<object> ExecuteAsync(SpaceshipApiClient client, GetSettings settings)
    {
        var result = await client.GetAsync($"/sellerhub/safepay-transactions/{settings.TransactionId}");
        return ToObject(result);
    }
}

public sealed class CreateSettings : GlobalSettings
{
    [CommandOption("--file <FILE>")]
    [Description("JSON file with the transaction (or pipe via stdin)")]
    public string? File { get; set; }
}

[Description("Create a SafePay (escrow) transaction")]
public sealed class CreateCommand : SpaceshipCommand<CreateSettings>
{
    protected override async Task<object> ExecuteAsync(SpaceshipApiClient client, CreateSettings settings)
    {
        var body = await JsonInput.ReadAsync(settings.File,
            "{\"domainName\": \"...\", \"initiatedBy\": \"seller\", \"basePrice\": {...}, \"type\": \"buyNow\", \"feePercentageShare\": {...}}");
        var result = await client.PostAsync("/sellerhub/safepay-transactions", ToObject(body));
        return ToObject(result);
    }
}
