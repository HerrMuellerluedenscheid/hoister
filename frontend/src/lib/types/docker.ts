export type InspectionType = {
    Id: string;
    State: {
        Status: string;
        Running: boolean;
        Paused: boolean;
        Error: string;
        FinishedAt: string;  // ISO 8601 date string
        StartedAt: string;   // ISO 8601 date string
    };
    Config: {
        Image: string;
        Hostname: string;
        User: string;
        Labels: Record<string, string>;
    };
    Labels: Record<string, string>;
};
