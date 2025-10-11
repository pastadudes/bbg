use crate::Error;
use serde::Deserialize;

// raw API structures (keep close to API)
#[derive(Deserialize, Debug)]
struct ApiJob {
    pub company_name: String,
    pub title: String,
    pub description: String,
    pub remote: bool,
    pub tags: Vec<String>,
    pub url: String,
    pub job_types: Vec<String>,
    pub location: String,
    pub created_at: u64,
}

#[derive(Deserialize, Debug)]
struct JobApiResponse {
    pub data: Vec<ApiJob>,
}

#[derive(Debug, Clone)]
pub struct Job {
    pub title: String,
    pub company: String,
    pub location: String,
    pub description: String,
    pub remote: bool,
    pub tags: Vec<String>,
    pub url: String,
    pub job_types: Vec<String>,
    pub created_at: u64,
}

impl From<ApiJob> for Job {
    fn from(api_job: ApiJob) -> Self {
        Self {
            title: api_job.title,
            company: api_job.company_name,
            location: api_job.location,
            description: nanohtml2text::html2text(&api_job.description),
            remote: api_job.remote,
            tags: api_job.tags,
            url: api_job.url,
            job_types: api_job.job_types,
            created_at: api_job.created_at,
        }
    }
}

#[derive(Debug, Clone)]
pub struct JobListings {
    pub jobs: Vec<Job>,
}

impl JobListings {
    pub async fn fetch() -> Result<Self, Error> {
        let client = reqwest::ClientBuilder::new()
            .user_agent("contact@pastaya.net if im being too spammy")
            .build()?;
        let response = client
            .get("https://arbeitnow.com/api/job-board-api")
            .send()
            .await?;
        let api_response: JobApiResponse = response.json().await?;

        Ok(Self {
            jobs: api_response.data.into_iter().map(Job::from).collect(),
        })
    }

    pub fn take(self, count: usize) -> Self {
        Self {
            jobs: self.jobs.into_iter().take(count).collect(),
        }
    }

    #[cfg(feature = "discord")]
    pub fn to_embed(&self) -> serenity::all::CreateEmbed {
        use serenity::all::CreateEmbed;
        let mut description = String::new();

        for (index, job) in self.jobs.iter().enumerate() {
            let truncated_desc = if job.description.chars().count() > 150 {
                format!(
                    "{}...",
                    job.description.chars().take(150).collect::<String>()
                )
            } else {
                job.description.clone()
            };

            description.push_str(&format!(
                "**{}. {}**\n\
                 **company**: {}\n\
                 **location**: {}\n\
                 **description**: {}\n\
                 **remote**: {}\n\
                 **tags**: {}\n\
                 **job types**: {}\n\
                 **posted** <t:{}:R>\n\
                 [view job]({})\n\n",
                index + 1,
                job.title,
                job.company,
                job.location,
                truncated_desc,
                if job.remote { "yes" } else { "no" },
                job.tags.join(", "),
                job.job_types.join(", "),
                job.created_at,
                job.url
            ));
        }

        CreateEmbed::default()
            .title("latest jobs (top 5)")
            .description(description)
            .color(serenity::all::colours::branding::FUCHSIA)
    }
}
