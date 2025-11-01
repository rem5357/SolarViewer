# PowerShell script to setup PostgreSQL with PostGIS for StellarForge
# Password: Beta5357

param(
    [string]$PostgresPassword = "Beta5357",
    [string]$DatabaseName = "stellarforge",
    [string]$PostgresUser = "postgres",
    [string]$Host = "localhost",
    [int]$Port = 5432
)

Write-Host "StellarForge Database Setup Script" -ForegroundColor Green
Write-Host "===================================" -ForegroundColor Green

# Check if PostgreSQL is installed
try {
    $pgVersion = & psql --version 2>$null
    if ($pgVersion) {
        Write-Host "PostgreSQL found: $pgVersion" -ForegroundColor Cyan
    }
} catch {
    Write-Host "PostgreSQL not found. Please install PostgreSQL first." -ForegroundColor Red
    Write-Host "Download from: https://www.postgresql.org/download/windows/" -ForegroundColor Yellow
    exit 1
}

# Set PGPASSWORD environment variable for non-interactive authentication
$env:PGPASSWORD = $PostgresPassword

# Function to execute SQL
function Execute-SQL {
    param(
        [string]$Query,
        [string]$Database = "postgres"
    )

    $result = & psql -h $Host -p $Port -U $PostgresUser -d $Database -c $Query 2>&1
    return $result
}

# Step 1: Check connection
Write-Host "`nStep 1: Testing PostgreSQL connection..." -ForegroundColor Yellow
$testResult = Execute-SQL "SELECT version();"
if ($testResult -match "PostgreSQL") {
    Write-Host "Connection successful!" -ForegroundColor Green
} else {
    Write-Host "Connection failed. Please check your PostgreSQL installation and password." -ForegroundColor Red
    Write-Host "Error: $testResult" -ForegroundColor Red
    exit 1
}

# Step 2: Create database
Write-Host "`nStep 2: Creating database '$DatabaseName'..." -ForegroundColor Yellow
$createDbResult = Execute-SQL "CREATE DATABASE $DatabaseName;"
if ($createDbResult -match "ERROR.*already exists") {
    Write-Host "Database already exists. Continuing..." -ForegroundColor Cyan
} elseif ($createDbResult -match "CREATE DATABASE") {
    Write-Host "Database created successfully!" -ForegroundColor Green
} else {
    Write-Host "Error creating database: $createDbResult" -ForegroundColor Red
}

# Step 3: Install PostGIS extension
Write-Host "`nStep 3: Installing PostGIS extension..." -ForegroundColor Yellow

# Check if PostGIS is available
$checkPostGIS = Execute-SQL "SELECT * FROM pg_available_extensions WHERE name = 'postgis';" $DatabaseName
if ($checkPostGIS -match "postgis") {
    # Install PostGIS
    $installResult = Execute-SQL "CREATE EXTENSION IF NOT EXISTS postgis;" $DatabaseName
    $installTopology = Execute-SQL "CREATE EXTENSION IF NOT EXISTS postgis_topology;" $DatabaseName

    # Verify installation
    $versionCheck = Execute-SQL "SELECT PostGIS_Version();" $DatabaseName
    if ($versionCheck -match "POSTGIS") {
        Write-Host "PostGIS installed successfully!" -ForegroundColor Green
        Write-Host "PostGIS Version: $($versionCheck -split '\n' | Where-Object { $_ -match 'POSTGIS' })" -ForegroundColor Cyan
    } else {
        Write-Host "PostGIS installation verification failed" -ForegroundColor Red
    }
} else {
    Write-Host "PostGIS extension not available. Please install PostGIS for PostgreSQL." -ForegroundColor Red
    Write-Host "Download from: http://postgis.net/install/" -ForegroundColor Yellow
    Write-Host "Or install via Stack Builder if using EnterpriseDB PostgreSQL" -ForegroundColor Yellow
    exit 1
}

# Step 4: Run StellarForge migrations
Write-Host "`nStep 4: Running StellarForge migrations..." -ForegroundColor Yellow
Write-Host "Please run the following Rust command to initialize the database:" -ForegroundColor Cyan
Write-Host ""
Write-Host "  cargo run --bin stellarforge -- init" -ForegroundColor White
Write-Host ""
Write-Host "Or with the database URL:" -ForegroundColor Cyan
Write-Host ""
Write-Host "  cargo run --bin stellarforge -- --database-url ""postgresql://${PostgresUser}:${PostgresPassword}@${Host}:${Port}/${DatabaseName}"" init" -ForegroundColor White
Write-Host ""

# Step 5: Create .env file for convenience
Write-Host "`nStep 5: Creating .env file..." -ForegroundColor Yellow
$envContent = @"
# StellarForge Database Configuration
DATABASE_URL=postgresql://${PostgresUser}:${PostgresPassword}@${Host}:${Port}/${DatabaseName}
"@

$envContent | Out-File -FilePath ".env.stellarforge" -Encoding UTF8
Write-Host ".env.stellarforge file created" -ForegroundColor Green

# Summary
Write-Host "`n===================================" -ForegroundColor Green
Write-Host "Setup Complete!" -ForegroundColor Green
Write-Host "===================================" -ForegroundColor Green
Write-Host ""
Write-Host "Database: $DatabaseName" -ForegroundColor Cyan
Write-Host "Host: ${Host}:${Port}" -ForegroundColor Cyan
Write-Host "User: $PostgresUser" -ForegroundColor Cyan
Write-Host ""
Write-Host "Next steps:" -ForegroundColor Yellow
Write-Host "1. Run 'cargo run --bin stellarforge -- init' to initialize tables" -ForegroundColor White
Write-Host "2. Run 'cargo run --bin stellarforge -- session create --name ""My Galaxy""' to create a session" -ForegroundColor White
Write-Host "3. Start importing or generating stellar data!" -ForegroundColor White

# Clear password from environment
Remove-Item Env:PGPASSWORD